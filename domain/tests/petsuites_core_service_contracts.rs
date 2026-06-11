use domain::{entities, money, operations};
use uuid::Uuid;

#[test]
fn boarding_contract_encodes_capacity_stay_payment_housekeeping_handoff_and_upsell_rules() {
    let contract = operations::boarding::Contract::builder()
        .capacity(operations::boarding::CapacityPlan::new(
            operations::boarding::RoomInventory::try_new(48).unwrap(),
            operations::boarding::RoomAvailability::Limited,
        ))
        .arrival_window(
            operations::boarding::ServiceWindow::new(
                operations::boarding::HourOfDay::try_new(7).unwrap(),
                operations::boarding::HourOfDay::try_new(18).unwrap(),
            )
            .unwrap(),
        )
        .departure_window(
            operations::boarding::ServiceWindow::new(
                operations::boarding::HourOfDay::try_new(7).unwrap(),
                operations::boarding::HourOfDay::try_new(12).unwrap(),
            )
            .unwrap(),
        )
        .minimum_stay(operations::boarding::MinimumStay::new(
            operations::boarding::StayNights::try_new(2).unwrap(),
            operations::boarding::MinimumStayReason::HolidayPeak,
        ))
        .cancellation(operations::boarding::CancellationPolicy::new(
            operations::boarding::NoticeHours::try_new(48).unwrap(),
            operations::boarding::CancellationPenalty::ForfeitDeposit,
        ))
        .deposit(operations::boarding::DepositRule::Required {
            amount: money::Money::new(
                money::MinorUnits::try_new(5_000).unwrap(),
                money::Currency::Usd,
            ),
        })
        .payment(operations::boarding::PaymentTiming::DueAtCheckout)
        .housekeeping(operations::boarding::HousekeepingCadence::DailyRoomReset)
        .handoff(operations::boarding::HandoffRequirement::ArrivalCareReview)
        .upsells(vec![
            operations::boarding::Upsell::ExitBath,
            operations::boarding::Upsell::TrainingSession,
        ])
        .build();

    assert_eq!(contract.capacity.room_inventory().get(), 48);
    assert!(contract.requires_deposit_collection());
    assert_eq!(contract.minimum_stay.nights().get(), 2);
    assert!(operations::boarding::RoomInventory::try_new(0).is_err());
    assert!(operations::boarding::StayNights::try_new(0).is_err());
    assert!(operations::boarding::NoticeHours::try_new(0).is_err());
    assert!(operations::boarding::HourOfDay::try_new(24).is_err());
    assert!(
        operations::boarding::ServiceWindow::new(
            operations::boarding::HourOfDay::try_new(18).unwrap(),
            operations::boarding::HourOfDay::try_new(7).unwrap(),
        )
        .is_err()
    );
}

#[test]
fn boarding_capacity_policy_never_uses_cat_condo_for_dog_request() {
    let snapshot = operations::boarding::capacity::Snapshot::new(vec![
        operations::boarding::capacity::NightlySegmentSnapshot::new(
            operations::boarding::accommodation::Kind::CatCondo,
            operations::boarding::capacity::RoomCount::try_new(4).unwrap(),
            operations::boarding::capacity::RoomCount::try_new(0).unwrap(),
        ),
    ])
    .unwrap();
    let request = operations::boarding::capacity::Request::new(
        entities::LocationId(Uuid::nil()),
        entities::Species::Dog,
        operations::boarding::accommodation::Preference::Specific(
            operations::boarding::accommodation::Kind::CatCondo,
        ),
    );

    let decision = operations::boarding::capacity::Policy.evaluate(&request, &snapshot);

    assert!(matches!(
        decision,
        operations::boarding::capacity::Decision::Deny {
            reason: operations::boarding::capacity::DenialReason::SpeciesAccommodationMismatch,
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
    let rule = operations::boarding::DepositRule::Required {
        amount: money::Money::new(
            money::MinorUnits::try_new(2_500).unwrap(),
            money::Currency::Usd,
        ),
    };
    let decision = operations::boarding::deposit::Policy::new(
        rule,
        operations::boarding::PaymentTiming::DueAtBooking,
    )
    .readiness_for_confirmation(None);

    assert!(matches!(
        decision,
        operations::boarding::deposit::ConfirmationReadiness::Blocked {
            blocker: operations::boarding::deposit::Blocker::DepositRequired,
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
    let rule = operations::boarding::DepositRule::Required {
        amount: amount.clone(),
    };
    let paid = domain::payment::Deposit::paid(
        amount,
        domain::payment::PaymentReference::try_new("gingr-pay-123").unwrap(),
    );

    let decision = operations::boarding::deposit::Policy::new(
        rule,
        operations::boarding::PaymentTiming::DueAtBooking,
    )
    .readiness_for_confirmation(Some(&paid));

    assert_eq!(
        decision,
        operations::boarding::deposit::ConfirmationReadiness::Ready
    );
}

#[test]
fn boarding_care_policy_flags_missing_feeding_instruction_for_staff_review() {
    let care_profile = entities::CareProfile::default();
    let plan = operations::boarding::care::Policy
        .plan_for_pet(entities::PetId(Uuid::nil()), &care_profile);

    assert_eq!(
        plan.readiness(),
        operations::boarding::care::Readiness::Blocked
    );
    assert!(
        plan.gates()
            .contains(&operations::boarding::care::ReviewGate::new(
                operations::boarding::care::GateReason::MissingFeedingInstruction,
                domain::policy::ReviewGate::MedicalDocumentReview,
            ))
    );
}

#[test]
fn boarding_upsell_policy_recommends_exit_bath_only_when_eligible_and_not_care_unsafe() {
    let mut care_profile = entities::CareProfile::default();
    care_profile
        .allergies
        .push(domain::care::AllergyName::try_new("sensitive shampoo").unwrap());

    let recommendation = operations::boarding::upsell::Policy.evaluate_exit_bath(
        entities::ReservationId(Uuid::nil()),
        entities::PetId(Uuid::nil()),
        &care_profile,
    );

    assert!(matches!(
        recommendation.eligibility,
        operations::boarding::upsell::Eligibility::NeedsStaffReview {
            gate: domain::policy::ReviewGate::MedicalDocumentReview,
            reason: operations::boarding::upsell::ReviewReason::CareSafetyAmbiguity,
        }
    ));
    assert_eq!(
        recommendation.customer_offer_gate(),
        Some(domain::policy::ReviewGate::CustomerMessageApproval)
    );
}

#[test]
fn daycare_contract_encodes_attendance_packages_ratios_groups_incidents_and_eligibility() {
    let contract = operations::daycare::Contract::builder()
        .attendance(operations::daycare::AttendancePolicy::ReservationRequired)
        .package(operations::daycare::PackagePolicy::PrepaidPasses {
            visits: operations::daycare::PackageVisits::try_new(10).unwrap(),
        })
        .ratio(operations::daycare::StaffPetRatio::new(
            operations::daycare::StaffCount::try_new(1).unwrap(),
            operations::daycare::PetCount::try_new(12).unwrap(),
        ))
        .group_assignment(operations::daycare::GroupAssignmentRule::TemperamentAndSizeMatched)
        .incident(operations::daycare::IncidentPolicy::ManagerReviewAndCustomerNotice)
        .eligibility(vec![
            operations::daycare::EligibilityRequirement::TemperamentAssessment,
            operations::daycare::EligibilityRequirement::VaccinesCurrent,
        ])
        .build();

    assert_eq!(contract.ratio.pets_per_staff().get(), 12);
    assert!(contract.requires_staff_review_before_group_play());
    assert!(operations::daycare::PackageVisits::try_new(0).is_err());
    assert!(operations::daycare::StaffCount::try_new(0).is_err());
    assert!(operations::daycare::PetCount::try_new(0).is_err());
}

#[test]
fn daycare_service_variants_preserve_group_boarding_plus_room_and_cat_care_modes() {
    assert_eq!(
        operations::daycare::ServiceVariant::AllDayPlay.care_mode(),
        operations::daycare::CareMode::DogGroupPlay
    );
    assert_eq!(
        operations::daycare::ServiceVariant::DayBoarding.care_mode(),
        operations::daycare::CareMode::DogIndividualDayBoarding
    );
    assert_eq!(
        operations::daycare::ServiceVariant::DayPlayPlusRoom.care_mode(),
        operations::daycare::CareMode::DogHybridPlayAndRoom
    );
    assert_eq!(
        operations::daycare::ServiceVariant::CatIndividualPlaytime.care_mode(),
        operations::daycare::CareMode::CatIndividualEnrichment
    );
}

#[test]
fn daycare_group_play_eligibility_routes_intact_dogs_to_behavior_review_not_ready() {
    let evidence = operations::daycare::eligibility::Evidence::builder()
        .pet_id(entities::PetId(Uuid::nil()))
        .species(entities::Species::Dog)
        .service(operations::daycare::ServiceVariant::AllDayPlay)
        .temperament(operations::daycare::eligibility::TemperamentAssessmentFreshness::Current)
        .vaccines(operations::daycare::eligibility::VaccineReadiness::Current)
        .spay_neuter(entities::SpayNeuterStatus::Intact)
        .incident(operations::daycare::incident::Restriction::None)
        .staff_coverage(operations::daycare::coverage::Decision::Sufficient)
        .build();

    let decision = operations::daycare::eligibility::GroupPlayPolicy.evaluate(&evidence);

    assert!(matches!(
        decision,
        operations::daycare::eligibility::GroupPlayDecision::NeedsStaffReview {
            reason: operations::daycare::eligibility::ReviewReason::SpayNeuterStatusRequiresReview,
            gate: domain::policy::ReviewGate::BehaviorReview,
        }
    ));
}

#[test]
fn daycare_staff_coverage_policy_rejects_rosters_that_exceed_contract_ratio() {
    let contract_ratio = operations::daycare::StaffPetRatio::new(
        operations::daycare::StaffCount::try_new(1).unwrap(),
        operations::daycare::PetCount::try_new(8).unwrap(),
    );
    let roster = operations::daycare::coverage::RosterSnapshot::new(
        operations::daycare::StaffCount::try_new(2).unwrap(),
        operations::daycare::PetCount::try_new(17).unwrap(),
    );

    let decision = operations::daycare::coverage::Policy.evaluate(&roster, contract_ratio);

    assert_eq!(
        decision,
        operations::daycare::coverage::Decision::Insufficient {
            reason: operations::daycare::coverage::InsufficiencyReason::RatioExceeded,
            gate: domain::policy::ReviewGate::ManagerApproval,
        }
    );
}

#[test]
fn daycare_assignment_requires_group_play_eligibility_and_staff_coverage() {
    let request = operations::daycare::assignment::Request::builder()
        .pet_id(entities::PetId(Uuid::nil()))
        .service(operations::daycare::ServiceVariant::HalfDayPlay)
        .eligibility(
            operations::daycare::eligibility::GroupPlayDecision::Eligible {
                basis: operations::daycare::eligibility::EligibleBasis::CurrentEvidence,
            },
        )
        .coverage(operations::daycare::coverage::Decision::Insufficient {
            reason: operations::daycare::coverage::InsufficiencyReason::RatioExceeded,
            gate: domain::policy::ReviewGate::ManagerApproval,
        })
        .playgroup(
            operations::daycare::assignment::PlaygroupId::try_new(" small-dogs-am ").unwrap(),
        )
        .build();

    let decision = operations::daycare::assignment::Service.assign(request);

    assert_eq!(
        decision,
        operations::daycare::assignment::Decision::Waitlist {
            reason: operations::daycare::assignment::WaitlistReason::StaffCoverageInsufficient,
            gate: domain::policy::ReviewGate::ManagerApproval,
        }
    );
}

#[test]
fn daycare_incident_policy_suspends_group_play_for_safety_incidents_until_manager_review() {
    let disposition = operations::daycare::incident::Policy.classify(
        entities::PetId(Uuid::nil()),
        operations::daycare::incident::Severity::SuspendGroupPlay,
    );

    assert_eq!(
        disposition.restriction(),
        operations::daycare::incident::Restriction::SuspendedPendingManagerReview {
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
    let recurrence = operations::daycare::attendance::Recurrence::new(
        operations::daycare::attendance::DateRange::new(
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        )
        .unwrap(),
        operations::daycare::attendance::AttendanceDays::try_new(vec![
            chrono::Weekday::Mon,
            chrono::Weekday::Wed,
            chrono::Weekday::Fri,
        ])
        .unwrap(),
    );

    let dates = operations::daycare::attendance::Materializer.materialize(
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
    let evidence = operations::daycare::package_opportunity::Evidence::builder()
        .customer_id(entities::CustomerId(Uuid::nil()))
        .pet_id(entities::PetId(Uuid::nil()))
        .attendance_visits(operations::daycare::package_opportunity::AttendanceVisitCount::new(12))
        .eligibility(
            operations::daycare::package_opportunity::CareEligibility::BlockedBySafetyReview,
        )
        .package_state(operations::daycare::package_opportunity::PackageState::PayPerVisit)
        .payment_state(operations::daycare::package_opportunity::PaymentState::Current)
        .build();

    let decision = operations::daycare::package_opportunity::Policy.classify(&evidence);

    assert_eq!(
        decision,
        operations::daycare::package_opportunity::Decision::Suppressed {
            reason: operations::daycare::package_opportunity::SuppressionReason::SafetyOrCareReviewRequired,
            gate: domain::policy::ReviewGate::BehaviorReview,
        }
    );
}

#[test]
fn daycare_front_desk_throughput_routes_ready_pets_to_fast_lane_without_customer_send() {
    let context = operations::daycare::front_desk::ReadinessContext::builder()
        .reservation_id(entities::ReservationId(Uuid::nil()))
        .service(operations::daycare::ServiceVariant::AllDayPlay)
        .eligibility(
            operations::daycare::front_desk::EligibilityReadiness::GroupPlay(
                operations::daycare::eligibility::GroupPlayDecision::Eligible {
                    basis: operations::daycare::eligibility::EligibleBasis::CurrentEvidence,
                },
            ),
        )
        .coverage(operations::daycare::coverage::Decision::Sufficient)
        .care(operations::daycare::front_desk::CareReadiness::Ready)
        .package(operations::daycare::front_desk::PackageReadiness::Ready)
        .customer_message(
            operations::daycare::front_desk::CustomerMessageReadiness::NoMessageNeeded,
        )
        .build();

    let decision = operations::daycare::front_desk::ThroughputPolicy.evaluate(&context);
    let ticket = operations::daycare::front_desk::QueueTicket::new(
        operations::daycare::front_desk::QueuePosition::try_new(1).unwrap(),
        decision.clone(),
    );

    assert_eq!(
        decision,
        operations::daycare::front_desk::ReadinessDecision::ReadyToCheckIn
    );
    assert_eq!(
        ticket.lane(),
        operations::daycare::front_desk::QueueLane::FastLane
    );
    assert_eq!(decision.customer_message_gate(), None);
}

#[test]
fn grooming_contract_encodes_calendar_estimates_no_shows_rebooking_reminders_and_history() {
    let estimate = operations::grooming::BreedCoatTimeEstimate::new(
        operations::grooming::BreedCategory::Doodle,
        operations::grooming::CoatCondition::Matted,
        operations::grooming::AppointmentMinutes::try_new(180).unwrap(),
    );
    let contract = operations::grooming::Contract::builder()
        .calendar(operations::grooming::CalendarPolicy::GroomerSpecific)
        .time_estimates(vec![estimate])
        .no_show(operations::grooming::NoShowPolicy::RequireDepositForRebooking)
        .rebooking(operations::grooming::RebookingCadence::EveryWeeks(
            operations::grooming::CadenceWeeks::try_new(6).unwrap(),
        ))
        .reminders(vec![
            operations::grooming::ReminderRule::FortyEightHoursBefore,
            operations::grooming::ReminderRule::MorningOf,
        ])
        .history(operations::grooming::HistoryRequirement::KeepStyleNotesAndPhotos)
        .build();

    assert_eq!(contract.time_estimates[0].minutes().get(), 180);
    assert!(contract.requires_deposit_after_no_show());
    assert!(operations::grooming::AppointmentMinutes::try_new(0).is_err());
    assert!(operations::grooming::CadenceWeeks::try_new(0).is_err());
}

#[test]
fn grooming_duration_estimate_requires_positive_minutes_and_explains_basis() {
    let estimate = operations::grooming::EstimationPolicy.estimate(
        operations::grooming::EstimationRequest::builder()
            .pet_id(entities::PetId(Uuid::nil()))
            .service(operations::GroomingService::FullGroom)
            .breed(operations::grooming::BreedCategory::Doodle)
            .coat(operations::grooming::CoatCondition::Maintained)
            .build(),
        &[],
        &operations::grooming::Contract::standard_petsuites(),
    );

    assert_eq!(estimate.minutes().get(), 180);
    assert_eq!(
        estimate.basis(),
        operations::grooming::EstimateBasis::BreedCoatPolicy
    );
    assert_eq!(
        estimate.review(),
        operations::grooming::ReviewRequirement::None
    );
    assert_eq!(estimate.calendar_execution_gate(), None);
}

#[test]
fn matted_or_sensitive_coat_estimate_requires_staff_review_before_auto_scheduling() {
    let estimate = operations::grooming::EstimationPolicy.estimate(
        operations::grooming::EstimationRequest::builder()
            .pet_id(entities::PetId(Uuid::nil()))
            .service(operations::GroomingService::FullGroom)
            .breed(operations::grooming::BreedCategory::Doodle)
            .coat(operations::grooming::CoatCondition::Matted)
            .build(),
        &[],
        &operations::grooming::Contract::standard_petsuites(),
    );

    assert_eq!(
        estimate.review(),
        operations::grooming::ReviewRequirement::GroomerReview
    );
    assert_eq!(
        estimate.calendar_execution_gate(),
        Some(domain::policy::ReviewGate::ManagerApproval)
    );
}

#[test]
fn grooming_rebooking_cadence_accepts_two_to_eight_week_ordinary_window() {
    assert!(operations::grooming::OrdinaryCadenceWeeks::try_new(2).is_ok());
    assert!(operations::grooming::OrdinaryCadenceWeeks::try_new(8).is_ok());
    assert!(operations::grooming::OrdinaryCadenceWeeks::try_new(1).is_err());
    assert!(operations::grooming::OrdinaryCadenceWeeks::try_new(9).is_err());
}

#[test]
fn grooming_no_show_policy_requires_deposit_or_manager_review_for_repeat_no_show() {
    let decision = operations::grooming::no_show::Policy::new(
        operations::grooming::NoShowPolicy::RequireDepositForRebooking,
    )
    .evaluate(
        entities::CustomerId(Uuid::nil()),
        entities::PetId(Uuid::nil()),
        operations::grooming::no_show::History::new(
            operations::grooming::no_show::NoShowCount::try_new(2).unwrap(),
            operations::grooming::no_show::LateCancelCount::try_new(1).unwrap(),
        ),
    );

    assert_eq!(
        decision,
        operations::grooming::no_show::Decision::DepositRequired {
            gate: domain::policy::ReviewGate::RefundOrDepositException,
        }
    );
}

#[test]
fn grooming_rebooking_policy_marks_pet_overdue_from_last_service_history_and_cadence() {
    let history_entry = operations::grooming::history::ServiceHistoryEntry::builder()
        .pet_id(entities::PetId(Uuid::nil()))
        .location_id(entities::LocationId(Uuid::nil()))
        .service(operations::GroomingService::FullGroom)
        .completed_on(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .outcome(operations::grooming::history::ServiceOutcome::Completed)
        .approval(
            operations::grooming::history::ApprovalState::ApprovedByGroomer {
                groomer_id: entities::StaffId::try_new("groomer-1").unwrap(),
            },
        )
        .build();

    let recommendation = operations::grooming::RebookingPolicy.recommend_from_history(
        entities::PetId(Uuid::nil()),
        &[history_entry],
        operations::grooming::RebookingCadence::EveryWeeks(
            operations::grooming::CadenceWeeks::try_new(6).unwrap(),
        ),
        chrono::NaiveDate::from_ymd_opt(2026, 2, 20).unwrap(),
    );

    assert_eq!(
        recommendation.status,
        operations::grooming::RebookingStatus::Overdue
    );
    assert_eq!(
        recommendation.rationale,
        operations::grooming::RebookingRationale::LastCompletedServiceCadence
    );
}

#[test]
fn grooming_reminder_plan_requires_customer_consent_before_member_facing_send() {
    let plan = operations::grooming::ReminderPolicy.plan(
        entities::CustomerId(Uuid::nil()),
        operations::grooming::ReminderKind::RebookingDue,
        operations::grooming::CommunicationConsent::NotGranted,
    );

    assert_eq!(
        plan.send_boundary(),
        operations::grooming::ReminderSendBoundary::SuppressedUntilConsent
    );
    assert_eq!(plan.customer_message_gate(), None);
}

#[test]
fn grooming_history_entry_separates_style_notes_from_care_or_medical_handling_refs() {
    let entry = operations::grooming::history::ServiceHistoryEntry::builder()
        .pet_id(entities::PetId(Uuid::nil()))
        .location_id(entities::LocationId(Uuid::nil()))
        .service(operations::GroomingService::MiniGroom)
        .completed_on(chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap())
        .outcome(operations::grooming::history::ServiceOutcome::Completed)
        .approval(
            operations::grooming::history::ApprovalState::ReviewRequired {
                gate: domain::policy::ReviewGate::MedicalDocumentReview,
            },
        )
        .style_notes(vec![
            operations::grooming::history::StyleNote::try_new(" teddy bear face ").unwrap(),
        ])
        .care_refs(vec![
            operations::grooming::history::CareReference::SensitiveSkinProduct,
        ])
        .build();

    assert_eq!(
        entry.style_notes()[0].clone().into_inner(),
        "teddy bear face"
    );
    assert_eq!(
        entry.care_refs(),
        &[operations::grooming::history::CareReference::SensitiveSkinProduct]
    );
    assert!(entry.requires_review());
}

#[test]
fn training_contract_encodes_program_curriculum_progress_outcomes_availability_packages_and_follow_up()
 {
    let contract = operations::training::Contract::builder()
        .program_duration(operations::training::ProgramDuration::Weeks(
            operations::training::DurationWeeks::try_new(3).unwrap(),
        ))
        .curriculum(vec![
            operations::training::CurriculumUnit::LooseLeashWalking,
            operations::training::CurriculumUnit::Recall,
        ])
        .progress(operations::training::ProgressTracking::SessionNotesAndMilestones)
        .outcomes(vec![
            operations::training::Outcome::CanineGoodCitizenReadiness,
        ])
        .trainer_availability(operations::training::TrainerAvailability::NamedTrainerRequired)
        .package(operations::training::PackagePolicy::MultiSessionPackage {
            sessions: operations::training::SessionCount::try_new(6).unwrap(),
        })
        .follow_up(operations::training::FollowUpCadence::AfterProgramCompletion)
        .build();

    assert!(contract.requires_named_trainer());
    assert!(contract.has_outcome(&operations::training::Outcome::CanineGoodCitizenReadiness));
    assert!(operations::training::DurationWeeks::try_new(0).is_err());
    assert!(operations::training::SessionCount::try_new(0).is_err());
}

#[test]
fn trainer_availability_waitlists_when_named_trainer_has_no_capacity() {
    let request = operations::training::availability::Request::builder()
        .enrollment_id(operations::training::EnrollmentId::try_new("enroll-123").unwrap())
        .pet_id(entities::PetId(Uuid::nil()))
        .program(operations::TrainingProgram::PrivateLesson)
        .requirement(operations::training::TrainerRequirement::NamedTrainer {
            trainer_id: entities::StaffId::try_new("trainer-7").unwrap(),
        })
        .capacity(operations::training::availability::CapacityDecision::Unavailable)
        .readiness(operations::training::EnrollmentReadiness::Ready)
        .build();

    let decision = operations::training::availability::Policy.evaluate(&request);

    assert_eq!(
        decision,
        operations::training::availability::Decision::Waitlist {
            reason: operations::training::availability::WaitlistReason::RequestedTrainerUnavailable,
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
    let report = operations::training::progress::Report::builder()
        .report_id(operations::training::ProgressReportId::try_new("progress-1").unwrap())
        .enrollment_id(operations::training::EnrollmentId::try_new("enroll-123").unwrap())
        .session_ref(operations::training::SessionRef::try_new("session-4").unwrap())
        .evidence(vec![operations::training::ProgressEvidence::TrainerNote {
            evidence_id: operations::training::EvidenceId::try_new("evidence-1").unwrap(),
            note: operations::training::ProgressNote::try_new(" recall improved with long line ")
                .unwrap(),
        }])
        .milestones(vec![operations::training::CurriculumProgress::new(
            operations::training::MilestoneId::try_new("recall").unwrap(),
            operations::training::MilestoneStatus::Introduced,
        )])
        .approval(operations::training::ApprovalState::Draft)
        .build()
        .unwrap();

    assert_eq!(
        report.parent_facing_boundary(),
        operations::training::MemberFacingBoundary::DraftRequiresApproval {
            gate: domain::policy::ReviewGate::CustomerMessageApproval,
        }
    );
    assert!(report.has_evidence());
}

#[test]
fn achieved_outcome_claim_requires_evidence_before_documentation_can_be_member_facing() {
    let rejected = operations::training::outcome::Claim::new(
        operations::training::Outcome::CanineGoodCitizenReadiness,
        operations::training::outcome::ClaimStatus::Achieved,
        vec![],
        vec![operations::training::MilestoneId::try_new("cgc-readiness").unwrap()],
    );

    assert_eq!(
        rejected,
        Err(operations::training::Error::OutcomeEvidenceRequired)
    );

    let claim = operations::training::outcome::Claim::new(
        operations::training::Outcome::CanineGoodCitizenReadiness,
        operations::training::outcome::ClaimStatus::Readiness,
        vec![operations::training::EvidenceId::try_new("rubric-1").unwrap()],
        vec![operations::training::MilestoneId::try_new("cgc-readiness").unwrap()],
    )
    .unwrap();
    let documentation = operations::training::outcome::Documentation::builder()
        .documentation_id(
            operations::training::OutcomeDocumentationId::try_new("outcome-1").unwrap(),
        )
        .enrollment_id(operations::training::EnrollmentId::try_new("enroll-123").unwrap())
        .pet_id(entities::PetId(Uuid::nil()))
        .location_id(entities::LocationId(Uuid::nil()))
        .claims(vec![claim])
        .review(operations::training::OutcomeReviewState::TrainerApproved {
            trainer_id: entities::StaffId::try_new("trainer-7").unwrap(),
        })
        .build()
        .unwrap();

    assert_eq!(
        documentation.member_facing_boundary(),
        operations::training::MemberFacingBoundary::DraftRequiresApproval {
            gate: domain::policy::ReviewGate::CustomerMessageApproval,
        }
    );
}

#[test]
fn training_package_ledger_exposes_remaining_sessions_without_callers_recomputing_counts() {
    let package_id = operations::training::package::Id::try_new("pkg-1").unwrap();
    let ledger = operations::training::package::Ledger::new(
        package_id.clone(),
        entities::CustomerId(Uuid::nil()),
        entities::PetId(Uuid::nil()),
        operations::training::PackagePolicy::MultiSessionPackage {
            sessions: operations::training::SessionCount::try_new(4).unwrap(),
        },
        vec![
            operations::training::package::LedgerEntry::Reserved {
                session_id: operations::training::TrainingSessionId::try_new("session-1").unwrap(),
            },
            operations::training::package::LedgerEntry::Consumed {
                session_id: operations::training::TrainingSessionId::try_new("session-2").unwrap(),
            },
        ],
    )
    .unwrap();

    assert_eq!(ledger.balance().remaining().get(), 2);
    assert_eq!(
        operations::training::package::Policy.decide_usage(&ledger),
        operations::training::package::UsageDecision::ReserveNextSession {
            package_id,
            remaining_after_reservation: operations::training::SessionBalance::new(1),
        }
    );
}

#[test]
fn follow_up_policy_creates_due_plan_for_after_each_session_with_progress_homework() {
    let plan = operations::training::follow_up::Policy.plan(
        operations::training::follow_up::Trigger::SessionCompleted {
            session_id: operations::training::TrainingSessionId::try_new("session-4").unwrap(),
        },
        operations::training::FollowUpCadence::AfterEachSession,
        operations::training::follow_up::EvidenceReadiness::ProgressAndHomeworkReady,
    );

    assert_eq!(
        plan.state(),
        operations::training::follow_up::State::DraftRequiresApproval {
            gate: domain::policy::ReviewGate::CustomerMessageApproval,
        }
    );
    assert_eq!(
        plan.purpose(),
        operations::training::follow_up::Purpose::ProgressUpdate
    );
}

#[test]
fn retail_contract_encodes_product_pos_inventory_recommendation_and_reorder_rules() {
    let contract = operations::retail::Contract::builder()
        .product(operations::retail::Product::new(
            operations::retail::Sku::try_new("  CALM-CARE-30  ").unwrap(),
            operations::RetailProductCategory::Supplement,
        ))
        .pos(operations::retail::PointOfSalePolicy::IntegratedWithReservationCheckout)
        .inventory(operations::retail::InventoryPolicy::Tracked {
            on_hand: operations::retail::UnitCount::try_new(8).unwrap(),
            reorder_at: operations::retail::UnitCount::try_new(10).unwrap(),
        })
        .recommendation(operations::retail::RecommendationRule::AnxietySupportAfterBoarding)
        .reorder(operations::retail::ReorderPolicy::AutoCreateManagerTask)
        .build();

    assert_eq!(contract.product.sku().clone().into_inner(), "CALM-CARE-30");
    assert!(contract.should_reorder());
    assert!(operations::retail::Sku::try_new("   ").is_err());
    assert!(operations::retail::UnitCount::try_new(0).is_err());
}

#[test]
fn retail_inventory_position_derives_available_units_and_rejects_over_reserved_stock() {
    let position = operations::retail::InventoryPosition::new(
        entities::LocationId(Uuid::nil()),
        operations::retail::Sku::try_new("CALM-CARE-30").unwrap(),
        operations::retail::OnHandUnits::new(8),
        operations::retail::ReservedUnits::new(3),
        operations::retail::UnitCount::try_new(10).unwrap(),
    )
    .unwrap();

    assert_eq!(
        position.available_units(),
        operations::retail::AvailableUnits::new(5)
    );
    assert_eq!(
        operations::retail::InventoryPosition::new(
            entities::LocationId(Uuid::nil()),
            operations::retail::Sku::try_new("CALM-CARE-30").unwrap(),
            operations::retail::OnHandUnits::new(2),
            operations::retail::ReservedUnits::new(3),
            operations::retail::UnitCount::try_new(10).unwrap(),
        ),
        Err(operations::retail::Error::ReservedUnitsExceedOnHand)
    );
}

#[test]
fn retail_reorder_policy_routes_below_threshold_stock_to_reviewable_staff_task() {
    let sku = operations::retail::Sku::try_new("CALM-CARE-30").unwrap();
    let position = operations::retail::InventoryPosition::new(
        entities::LocationId(Uuid::nil()),
        sku.clone(),
        operations::retail::OnHandUnits::new(4),
        operations::retail::ReservedUnits::new(0),
        operations::retail::UnitCount::try_new(10).unwrap(),
    )
    .unwrap();

    let decision = operations::retail::ReorderPolicy::AutoCreateManagerTask.evaluate(&position);

    assert_eq!(
        decision,
        operations::retail::ReorderDecision::CreateStaffTask {
            location_id: entities::LocationId(Uuid::nil()),
            sku,
            reason: operations::retail::ReorderReason::AtOrBelowThreshold,
            gate: domain::policy::ReviewGate::ManagerApproval,
        }
    );
}

#[test]
fn retail_pos_policy_requires_manager_approval_for_comps_discounts_and_refunds() {
    let offering = operations::retail::LocationOffering::builder()
        .location_id(entities::LocationId(Uuid::nil()))
        .product(operations::retail::Product::new(
            operations::retail::Sku::try_new("CALM-CARE-30").unwrap(),
            operations::RetailProductCategory::Supplement,
        ))
        .status(operations::retail::OfferingStatus::Active)
        .usage(operations::retail::ProductUsage::CustomerSellable)
        .pos(operations::retail::PointOfSalePolicy::IntegratedWithReservationCheckout)
        .inventory(operations::retail::InventoryPolicy::Tracked {
            on_hand: operations::retail::UnitCount::try_new(5).unwrap(),
            reorder_at: operations::retail::UnitCount::try_new(2).unwrap(),
        })
        .reorder(operations::retail::ReorderPolicy::ManualReview)
        .build();
    let request = operations::retail::SaleRequest::builder()
        .offering(offering)
        .quantity(operations::retail::SaleQuantity::try_new(1).unwrap())
        .source(operations::retail::SaleSource::ReservationCheckout {
            reservation_id: entities::ReservationId(Uuid::nil()),
        })
        .price_adjustment(operations::retail::PriceAdjustment::ManagerComp {
            reason: operations::retail::PriceExceptionReason::ComplaintRecovery,
        })
        .build();

    let decision =
        operations::retail::PointOfSalePolicy::IntegratedWithReservationCheckout.evaluate(&request);

    assert_eq!(
        decision,
        operations::retail::SaleLineDecision::ReviewRequired {
            reason: operations::retail::SaleReviewReason::PriceException,
            gate: domain::policy::ReviewGate::ManagerApproval,
        }
    );
}

#[test]
fn retail_recommendation_policy_routes_care_sensitive_supplement_candidates_to_staff_review() {
    let candidate = operations::retail::RecommendationCandidate::builder()
        .customer_id(entities::CustomerId(Uuid::nil()))
        .pet_id(entities::PetId(Uuid::nil()))
        .location_id(entities::LocationId(Uuid::nil()))
        .product(operations::retail::Product::new(
            operations::retail::Sku::try_new("CALM-CARE-30").unwrap(),
            operations::RetailProductCategory::Supplement,
        ))
        .reason(operations::retail::RecommendationReason::AnxietyOrStressSupport)
        .rationale(
            operations::retail::RecommendationRationale::try_new(
                "staff noted anxious boarding transition",
            )
            .unwrap(),
        )
        .care_sensitivity(operations::retail::CareSensitivity::SupplementOrDietReviewRequired)
        .inventory(operations::retail::InventoryAvailability::Available)
        .customer_preference(
            operations::retail::CustomerRetailPreference::AllowsRetailRecommendations,
        )
        .build();

    let decision = operations::retail::RecommendationPolicy.evaluate(&candidate);

    assert_eq!(
        decision,
        operations::retail::RecommendationDecision::StaffReviewRequired {
            reason: operations::retail::RecommendationReviewReason::CareSensitiveProduct,
            gate: domain::policy::ReviewGate::MedicalDocumentReview,
        }
    );
}

#[test]
fn retail_customer_copy_policy_forbids_medical_claims_in_customer_drafts() {
    let copy = operations::retail::CustomerSafeCopy::try_new(
        "Virbac CalmCare treats anxiety for boarding dogs",
    )
    .unwrap();

    let decision = operations::retail::CustomerCopyPolicy.evaluate(&copy);

    assert_eq!(
        decision,
        operations::retail::CustomerCopyDecision::Rejected {
            reason: operations::retail::CustomerCopyRejectionReason::MedicalClaim,
            gate: domain::policy::ReviewGate::CustomerMessageApproval,
        }
    );
}

#[test]
fn core_service_contract_groups_all_petsuites_lines_without_raw_field_flags() {
    let service_contracts = operations::CoreServiceContracts::builder()
        .location_id(entities::LocationId(uuid::Uuid::nil()))
        .boarding(operations::boarding::Contract::standard_petsuites())
        .daycare(operations::daycare::Contract::standard_petsuites())
        .grooming(operations::grooming::Contract::standard_petsuites())
        .training(operations::training::Contract::standard_petsuites())
        .retail(operations::retail::Contract::standard_petsuites())
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
