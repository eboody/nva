use domain::{
    agent, care, customer, daily_brief, entities, lead, location, money, operations, payment, pet,
    policy, portal, reputation, reservation, service::grooming, staff, temperament, workflow,
};

#[test]
fn pet_names_are_trimmed_non_empty_domain_values() {
    let name = pet::Name::try_new("  Moose  ").expect("valid trimmed pet name");

    assert_eq!(name.into_inner(), "Moose");
    assert!(pet::Name::try_new("   ").is_err());
}

#[test]
fn ordinary_domain_structs_have_compile_checked_bon_builders() {
    let spec = agent::Spec::builder()
        .name(agent::Name::try_new("booking-triage").unwrap())
        .purpose(
            agent::Purpose::try_new("Evaluate reservation requests against deterministic policy.")
                .unwrap(),
        )
        .allowed_tools(vec![agent::ToolName::try_new("availability-read").unwrap()])
        .forbidden_actions(vec![
            agent::ForbiddenAction::try_new("invent availability").unwrap(),
        ])
        .default_review_gates(vec![policy::ReviewGate::ManagerApproval])
        .build();

    assert_eq!(spec.name.into_inner(), "booking-triage");
    assert_eq!(spec.allowed_tools.len(), 1);
}

#[test]
fn core_pet_entity_uses_validated_pet_name() {
    let pet = entities::Pet {
        id: entities::PetId(uuid::Uuid::nil()),
        customer_id: entities::CustomerId(uuid::Uuid::nil()),
        name: pet::Name::try_new("  Moose  ").unwrap(),
        species: entities::Species::Dog,
        birth_date: None,
        sex: None,
        spay_neuter_status: entities::SpayNeuterStatus::Neutered,
        temperament: entities::TemperamentProfile::default(),
        care_profile: entities::CareProfile::default(),
    };

    assert_eq!(pet.name.into_inner(), "Moose");
}

#[test]
fn care_profile_uses_semantic_care_and_medical_contracts() {
    let medication = entities::MedicationInstruction::builder()
        .name(care::MedicationName::try_new("  Fluoxetine  ").unwrap())
        .dose(care::MedicationDose::try_new("  10 mg  ").unwrap())
        .schedule(care::MedicationSchedule::try_new("  twice daily with meals  ").unwrap())
        .review_requirement(care::MedicationReviewRequirement::RequiresReview {
            reason: care::ReviewReason::try_new("  new medication on intake  ").unwrap(),
        })
        .build();

    let profile = entities::CareProfile {
        feeding_instructions: Some(
            care::FeedingInstruction::try_new("  half cup morning and evening  ").unwrap(),
        ),
        medications: vec![medication],
        allergies: vec![care::AllergyName::try_new("  chicken  ").unwrap()],
        medical_conditions: vec![care::MedicalConditionName::try_new("  diabetes  ").unwrap()],
        emergency_contact: Some(care::ContactRef::new(
            care::ContactName::try_new("  Ana Rivera  ").unwrap(),
        )),
        veterinarian_contact: Some(care::ContactRef::new(
            care::ContactName::try_new("  Pine Vet Clinic  ").unwrap(),
        )),
    };

    assert_eq!(
        profile.feeding_instructions.unwrap().into_inner(),
        "half cup morning and evening"
    );
    assert_eq!(
        profile.medications[0].name.clone().into_inner(),
        "Fluoxetine"
    );
    assert_eq!(profile.allergies[0].clone().into_inner(), "chicken");
    assert_eq!(
        profile.medical_conditions[0].clone().into_inner(),
        "diabetes"
    );
    assert!(care::FeedingInstruction::try_new("   ").is_err());
    assert!(care::AllergyName::try_new("   ").is_err());
    assert!(care::MedicalConditionName::try_new("   ").is_err());
    assert!(care::MedicationName::try_new("   ").is_err());
    assert!(care::MedicationDose::try_new("   ").is_err());
    assert!(care::MedicationSchedule::try_new("   ").is_err());
    assert!(care::ContactName::try_new("   ").is_err());
    assert!(care::ReviewReason::try_new("   ").is_err());
}

#[test]
fn care_medical_debug_output_redacts_sensitive_details() {
    let profile = entities::CareProfile {
        feeding_instructions: Some(care::FeedingInstruction::try_new("quiet room feed").unwrap()),
        medications: vec![
            entities::MedicationInstruction::builder()
                .name(care::MedicationName::try_new("Fluoxetine").unwrap())
                .dose(care::MedicationDose::try_new("10 mg").unwrap())
                .schedule(care::MedicationSchedule::try_new("twice daily").unwrap())
                .review_requirement(care::MedicationReviewRequirement::RequiresReview {
                    reason: care::ReviewReason::try_new("new medication").unwrap(),
                })
                .build(),
        ],
        allergies: vec![care::AllergyName::try_new("chicken").unwrap()],
        medical_conditions: vec![care::MedicalConditionName::try_new("diabetes").unwrap()],
        emergency_contact: Some(care::ContactRef::new(
            care::ContactName::try_new("Ana Rivera").unwrap(),
        )),
        veterinarian_contact: None,
    };

    let debug = format!("{profile:?}");

    for sensitive_detail in [
        "quiet room feed",
        "Fluoxetine",
        "10 mg",
        "twice daily",
        "new medication",
        "chicken",
        "diabetes",
        "Ana Rivera",
    ] {
        assert!(
            !debug.contains(sensitive_detail),
            "debug output leaked {sensitive_detail}: {debug}"
        );
    }
}

#[test]
fn temperament_profile_uses_semantic_observation_contracts() {
    let profile = entities::TemperamentProfile::builder()
        .group_play_observation(temperament::GroupPlayObservation::NeedsIntroAssessment)
        .people_orientation(temperament::PeopleOrientation::PeopleSeeking)
        .rating(temperament::TemperamentRating::NeedsStructure)
        .behavior_observations(vec![
            temperament::BehaviorObservation::BiteHistory,
            temperament::BehaviorObservation::Extension(
                temperament::BehaviorObservationLabel::try_new("  leash reactivity  ").unwrap(),
            ),
        ])
        .staff_notes(vec![
            temperament::StaffNote::try_new("  Needs slow introductions before playgroup.  ")
                .unwrap(),
        ])
        .build();

    assert!(profile.needs_staff_play_evaluation());

    assert_eq!(
        profile.behavior_observations[1],
        temperament::BehaviorObservation::Extension(
            temperament::BehaviorObservationLabel::try_new("leash reactivity").unwrap()
        )
    );
    assert_eq!(
        profile.staff_notes[0].clone().into_inner(),
        "Needs slow introductions before playgroup."
    );
    assert!(temperament::StaffNote::try_new("   ").is_err());
    assert!(temperament::BehaviorObservationLabel::try_new("   ").is_err());
}

#[test]
fn temperament_debug_output_redacts_staff_notes() {
    let profile = entities::TemperamentProfile::builder()
        .staff_notes(vec![
            temperament::StaffNote::try_new("Owner reported bite incident on 2025-03-04.").unwrap(),
        ])
        .build();

    let debug = format!("{profile:?}");

    assert!(
        !debug.contains("Owner reported bite incident"),
        "debug output leaked staff note: {debug}"
    );

    let labeled = temperament::BehaviorObservation::Extension(
        temperament::BehaviorObservationLabel::try_new("thunderstorm trigger").unwrap(),
    );
    let debug = format!("{labeled:?}");

    assert!(
        !debug.contains("thunderstorm trigger"),
        "debug output leaked behavior observation label: {debug}"
    );
}

#[test]
fn location_policy_refs_use_policy_ids_not_raw_strings() {
    let refs = entities::LocationPolicyRefs {
        vaccine_policy_id: policy::Id::try_new(" vaccine-default ").unwrap(),
        deposit_policy_id: policy::Id::try_new("deposit-default").unwrap(),
        playgroup_policy_id: policy::Id::try_new("playgroup-default").unwrap(),
    };

    assert_eq!(refs.vaccine_policy_id.into_inner(), "vaccine-default");
}

#[test]
fn location_identity_uses_validated_location_values() {
    let location = entities::Location {
        id: entities::LocationId(uuid::Uuid::nil()),
        brand: entities::Brand::NeighborhoodPetResort {
            name: location::Name::try_new("  Pine Hollow Pet Resort  ").unwrap(),
        },
        name: location::Name::try_new("  Pine Hollow  ").unwrap(),
        timezone: location::Timezone::try_new("  America/New_York  ").unwrap(),
        capabilities: vec![entities::ServiceKind::Boarding],
        policies: entities::LocationPolicyRefs {
            vaccine_policy_id: policy::Id::try_new("vaccine-default").unwrap(),
            deposit_policy_id: policy::Id::try_new("deposit-default").unwrap(),
            playgroup_policy_id: policy::Id::try_new("playgroup-default").unwrap(),
        },
    };

    assert_eq!(location.name.into_inner(), "Pine Hollow");
    assert_eq!(location.timezone.into_inner(), "America/New_York");
    assert!(location::Name::try_new("   ").is_err());
    assert!(location::Timezone::try_new("   ").is_err());
}

#[test]
fn customer_contact_and_portal_identity_use_validated_values() {
    let customer = entities::Customer {
        id: entities::CustomerId(uuid::Uuid::nil()),
        full_name: customer::Name::try_new("  Ana Rivera  ").unwrap(),
        email: Some(customer::Email::try_new("  ana@example.com  ").unwrap()),
        mobile_phone: Some(customer::Phone::try_new("  +1 555 0100  ").unwrap()),
        preferred_contact: entities::ContactChannel::Email,
        portal_account: Some(entities::PortalAccountRef {
            provider: entities::PortalProvider::Gingr,
            external_customer_id: portal::CustomerId::try_new("  gingr-123  ").unwrap(),
        }),
    };

    assert_eq!(customer.full_name.into_inner(), "Ana Rivera");
    assert_eq!(customer.email.unwrap().into_inner(), "ana@example.com");
    assert_eq!(customer.mobile_phone.unwrap().into_inner(), "+1 555 0100");
    assert_eq!(
        customer
            .portal_account
            .unwrap()
            .external_customer_id
            .into_inner(),
        "gingr-123"
    );
    assert!(customer::Name::try_new("   ").is_err());
    assert!(customer::Email::try_new("   ").is_err());
    assert!(customer::Phone::try_new("   ").is_err());
    assert!(portal::CustomerId::try_new("   ").is_err());
}

#[test]
fn actor_refs_use_role_specific_identity_contracts() {
    let staff = entities::ActorRef::Staff {
        staff_id: entities::StaffId::try_new("  kennel-tech-17  ").unwrap(),
    };
    let manager = entities::ActorRef::Manager {
        manager_id: entities::ManagerId::try_new("  resort-manager-3  ").unwrap(),
    };
    let agent = entities::ActorRef::Agent {
        workflow: agent::Name::try_new("  booking-triage  ").unwrap(),
    };

    assert_eq!(
        staff,
        entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("kennel-tech-17").unwrap(),
        }
    );
    assert_eq!(
        manager,
        entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("resort-manager-3").unwrap(),
        }
    );
    assert_eq!(
        agent,
        entities::ActorRef::Agent {
            workflow: agent::Name::try_new("booking-triage").unwrap(),
        }
    );
    assert!(entities::StaffId::try_new("   ").is_err());
    assert!(entities::ManagerId::try_new("   ").is_err());
}

#[test]
fn audit_events_use_typed_subject_action_and_metadata_contracts() {
    let event = entities::AuditEvent {
        at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
        actor: entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("mgr-1").unwrap(),
        },
        subject: entities::AuditSubject::Reservation(entities::ReservationId(uuid::Uuid::nil())),
        action: entities::AuditAction::ReservationStatusSuggested,
        metadata: [(
            entities::AuditMetadataKey::try_new("  source_workflow  ").unwrap(),
            entities::AuditMetadataValue::try_new("  booking-triage  ").unwrap(),
        )]
        .into_iter()
        .collect(),
    };

    assert_eq!(
        event.subject,
        entities::AuditSubject::Reservation(entities::ReservationId(uuid::Uuid::nil()))
    );
    assert_eq!(
        event.metadata.keys().next().unwrap().clone().into_inner(),
        "source_workflow"
    );
    assert_eq!(
        event.metadata.values().next().unwrap().clone().into_inner(),
        "booking-triage"
    );
    assert!(entities::AuditMetadataKey::try_new("   ").is_err());
    assert!(entities::AuditMetadataValue::try_new("   ").is_err());
}

#[test]
fn money_deposit_age_and_add_on_contracts_quarantine_raw_primitives() {
    let amount = money::Money::new(
        money::MinorUnits::try_new(12_500).unwrap(),
        money::Currency::Usd,
    );
    assert_eq!(amount.minor_units().get(), 12_500);
    assert_eq!(amount.currency(), money::Currency::Usd);
    assert!(money::MinorUnits::try_new(0).is_err());

    let deposit = payment::Deposit::required(amount.clone());
    assert_eq!(deposit.status(), payment::DepositStatus::Required);
    assert_eq!(deposit.amount(), &amount);
    assert!(deposit.requires_collection());

    let paid = deposit.mark_paid(payment::PaymentReference::try_new("  stripe/pi_123  ").unwrap());
    assert_eq!(paid.status(), payment::DepositStatus::Paid);
    assert!(!paid.requires_collection());

    let threshold = reservation::AgeThreshold::new(
        reservation::MinimumAgeWeeks::try_new(16).unwrap(),
        reservation::AgePolicyReason::BoardingMinimum,
    );
    assert_eq!(threshold.minimum().get(), 16);
    assert_eq!(
        entities::HardStop::AgeBelowMinimumWeeks(threshold.clone()),
        entities::HardStop::AgeBelowMinimumWeeks(threshold)
    );
    assert!(reservation::MinimumAgeWeeks::try_new(0).is_err());

    let add_on =
        entities::AddOn::Other(reservation::AddOnLabel::try_new("  enrichment walk  ").unwrap());
    assert_eq!(
        add_on,
        entities::AddOn::Other(reservation::AddOnLabel::try_new("enrichment walk").unwrap())
    );
}

#[test]
fn semantic_scalars_reject_invalid_deserialized_primitives() {
    assert!(serde_json::from_str::<money::MinorUnits>("0").is_err());
    assert!(serde_json::from_str::<payment::PaymentReference>(r#""   ""#).is_err());
    assert!(serde_json::from_str::<reservation::MinimumAgeWeeks>("0").is_err());
    assert!(serde_json::from_str::<reservation::AddOnLabel>(r#""   ""#).is_err());
}

#[test]
fn reservation_hard_stops_carry_policy_semantics_not_raw_strings() {
    let missing_vaccine = entities::HardStop::MissingRequiredVaccine(
        policy::VaccineName::try_new("  Rabies  ").unwrap(),
    );
    let group_play_denial = entities::HardStop::IneligibleForGroupPlay(
        policy::PlayIneligibilityReason::BehaviorFlagsRequireReview,
    );

    assert_eq!(
        missing_vaccine,
        entities::HardStop::MissingRequiredVaccine(policy::VaccineName::try_new("Rabies").unwrap())
    );
    assert_eq!(
        group_play_denial,
        entities::HardStop::IneligibleForGroupPlay(
            policy::PlayIneligibilityReason::BehaviorFlagsRequireReview
        )
    );
}

#[test]
fn policy_surfaces_use_semantic_vaccine_and_play_eligibility_reasons() {
    let requirement = policy::VaccineRequirement {
        species: entities::Species::Dog,
        service: entities::ServiceKind::Boarding,
        vaccines: vec![policy::VaccineName::try_new("  Rabies  ").unwrap()],
        source_must_be_licensed_vet: true,
    };
    assert_eq!(requirement.vaccines[0].clone().into_inner(), "Rabies");
    assert!(policy::VaccineName::try_new("   ").is_err());

    let mut pet = entities::Pet {
        id: entities::PetId(uuid::Uuid::nil()),
        customer_id: entities::CustomerId(uuid::Uuid::nil()),
        name: pet::Name::try_new("Moose").unwrap(),
        species: entities::Species::Dog,
        birth_date: None,
        sex: None,
        spay_neuter_status: entities::SpayNeuterStatus::Intact,
        temperament: entities::TemperamentProfile::default(),
        care_profile: entities::CareProfile::default(),
    };
    let decision = policy::PlayEligibilityPolicy::decide(
        &policy::ConservativePlayEligibilityPolicy,
        &pet,
        &entities::ServiceKind::DayPlay,
    );
    assert_eq!(
        decision.eligibility,
        policy::PlayEligibility::Ineligible(
            policy::PlayIneligibilityReason::SpayNeuterStatusRequiresReview
        )
    );

    pet.spay_neuter_status = entities::SpayNeuterStatus::Neutered;
    let decision = policy::PlayEligibilityPolicy::decide(
        &policy::ConservativePlayEligibilityPolicy,
        &pet,
        &entities::ServiceKind::DayPlay,
    );
    assert_eq!(
        decision.eligibility,
        policy::PlayEligibility::Eligible(policy::PlayEligibilityReason::NoConservativeHardStop)
    );
}

#[test]
fn workflow_packets_results_and_actions_use_semantic_values() {
    let external_subject = workflow::WorkflowSubject::External {
        provider: workflow::external::Provider::try_new("  gingr  ").unwrap(),
        id: workflow::external::Id::try_new("  reservation-123  ").unwrap(),
    };

    match external_subject {
        workflow::WorkflowSubject::External { provider, id } => {
            assert_eq!(provider.into_inner(), "gingr");
            assert_eq!(id.into_inner(), "reservation-123");
        }
        _ => panic!("expected external subject"),
    }

    let result = workflow::WorkflowResult::<()> {
        status: workflow::WorkflowStatus::NeedsHumanReview,
        summary: workflow::Summary::try_new("  Vaccine date needs manager review.  ").unwrap(),
        structured_output: None,
        recommended_actions: vec![
            workflow::RecommendedAction::InternalTask {
                title: workflow::task::Title::try_new("  Review vaccine proof  ").unwrap(),
                body: workflow::task::Body::try_new("Confirm rabies expiration date.").unwrap(),
            },
            workflow::RecommendedAction::DraftMessage {
                channel: workflow::message::Channel::try_new("  email  ").unwrap(),
                body: workflow::message::Body::try_new("We need one more vaccine detail.").unwrap(),
            },
            workflow::RecommendedAction::UpdateStatus {
                target: workflow::status_update::Target::Reservation(
                    workflow::status_update::ReservationStatusUpdate {
                        status: entities::ReservationStatus::VaccinePending,
                        intent: workflow::status_update::TransitionIntent::RequestMedicalReview,
                        reason: workflow::status_update::Reason::try_new(
                            "Rabies certificate date is ambiguous.",
                        )
                        .unwrap(),
                    },
                ),
            },
        ],
        risk_flags: vec![workflow::RiskFlag::try_new("  vaccine ambiguity  ").unwrap()],
        verification: vec![workflow::VerificationNote::try_new("Matched policy gate.").unwrap()],
        human_review_reason: Some(
            workflow::ReviewReason::try_new("Medical document ambiguity").unwrap(),
        ),
    };

    assert_eq!(
        result.summary.into_inner(),
        "Vaccine date needs manager review."
    );
    assert_eq!(
        result.risk_flags[0].clone().into_inner(),
        "vaccine ambiguity"
    );
    match &result.recommended_actions[2] {
        workflow::RecommendedAction::UpdateStatus { target } => match target {
            workflow::status_update::Target::Reservation(update) => {
                assert_eq!(update.status, entities::ReservationStatus::VaccinePending);
                assert_eq!(
                    update.intent,
                    workflow::status_update::TransitionIntent::RequestMedicalReview
                );
                assert_eq!(
                    update.reason.clone().into_inner(),
                    "Rabies certificate date is ambiguous."
                );
            }
        },
        _ => panic!("expected reservation status update recommendation"),
    }
    assert!(workflow::Summary::try_new("   ").is_err());
    assert!(workflow::external::Provider::try_new("   ").is_err());
    assert!(workflow::task::Title::try_new("   ").is_err());
}

#[test]
fn workflow_status_updates_reject_arbitrary_entity_status_pairs() {
    let arbitrary_pair = serde_json::json!({
        "UpdateStatus": {
            "target": {
                "Customer": {
                    "status": "whatever-the-agent-invented",
                    "intent": "RequestMedicalReview",
                    "reason": "looks plausible"
                }
            }
        }
    });

    assert!(serde_json::from_value::<workflow::RecommendedAction>(arbitrary_pair).is_err());
}

#[test]
fn workflow_status_update_reasons_are_semantic_transition_contracts() {
    assert!(workflow::status_update::Reason::try_new("   ").is_err());
    assert!(workflow::status_update::Reason::try_new("a".repeat(501)).is_err());

    let update = workflow::status_update::ReservationStatusUpdate {
        status: entities::ReservationStatus::Waitlisted,
        intent: workflow::status_update::TransitionIntent::ApplyCapacityDecision,
        reason: workflow::status_update::Reason::try_new(
            "No suite is available for the requested dates.",
        )
        .unwrap(),
    };

    assert_eq!(update.status, entities::ReservationStatus::Waitlisted);
    assert_eq!(
        update.intent,
        workflow::status_update::TransitionIntent::ApplyCapacityDecision
    );
}

#[test]
fn nva_context_expands_daily_brief_contracts_for_resort_operations() {
    let brief = daily_brief::ResortDailyBrief {
        operating_day: daily_brief::ResortOperatingDay {
            location_id: entities::LocationId(uuid::Uuid::nil()),
            date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            snapshot_id: daily_brief::SnapshotId::try_new("  morning-brief-001  ").unwrap(),
        },
        sections: vec![
            daily_brief::DailyBriefSection::Occupancy(daily_brief::OccupancySnapshot {
                boarding_capacity: daily_brief::CapacityMetric::new(
                    daily_brief::CapacityBooked::new(42),
                    daily_brief::CapacityLimit::try_new(50).unwrap(),
                ),
                daycare_capacity: daily_brief::CapacityMetric::new(
                    daily_brief::CapacityBooked::new(31),
                    daily_brief::CapacityLimit::try_new(40).unwrap(),
                ),
                grooming_utilization: daily_brief::CapacityMetric::new(
                    daily_brief::CapacityBooked::new(6),
                    daily_brief::CapacityLimit::try_new(8).unwrap(),
                ),
                training_utilization: daily_brief::CapacityMetric::new(
                    daily_brief::CapacityBooked::new(2),
                    daily_brief::CapacityLimit::try_new(4).unwrap(),
                ),
            }),
            daily_brief::DailyBriefSection::Labor(daily_brief::LaborSnapshot {
                scheduled_staff_count: daily_brief::ScheduledStaffCount::new(7),
                labor_risk: daily_brief::LaborRisk::OnPlan,
            }),
            daily_brief::DailyBriefSection::PetCareWatchlist(vec![daily_brief::PetCareWatch {
                pet_id: entities::PetId(uuid::Uuid::nil()),
                reason: daily_brief::PetCareWatchReason::MedicationDue,
            }]),
            daily_brief::DailyBriefSection::RevenueOpportunities(vec![
                daily_brief::RevenueOpportunity {
                    customer_id: Some(entities::CustomerId(uuid::Uuid::nil())),
                    pet_id: Some(entities::PetId(uuid::Uuid::nil())),
                    service: entities::ServiceKind::Grooming,
                    opportunity: daily_brief::RevenueOpportunityKind::GroomingRebookingDue,
                },
            ]),
        ],
        recommended_actions: vec![daily_brief::Action::SuggestScheduleReview {
            risk: daily_brief::LaborRisk::Understaffed,
        }],
        risks: vec![daily_brief::Risk::LaborMismatch {
            risk: daily_brief::LaborRisk::Understaffed,
        }],
    };

    assert!(brief.has_manager_attention_required());
    let daily_brief::DailyBriefSection::Occupancy(occupancy) = &brief.sections[0] else {
        panic!("expected semantic occupancy section");
    };
    assert_eq!(
        occupancy.boarding_capacity.saturation_basis_points().get(),
        8400
    );
    assert!(daily_brief::CapacityLimit::try_new(0).is_err());
    assert_eq!(occupancy.boarding_capacity.booked().get(), 42);
    assert_eq!(occupancy.boarding_capacity.capacity().get(), 50);
    let daily_brief::DailyBriefSection::Labor(labor) = &brief.sections[1] else {
        panic!("expected semantic labor section");
    };
    assert_eq!(labor.scheduled_staff_count.get(), 7);
    assert_eq!(
        brief.operating_day.snapshot_id.into_inner(),
        "morning-brief-001"
    );
    assert!(daily_brief::SnapshotId::try_new("   ").is_err());
    assert!(operations::OperationalObservation::try_new("   ").is_err());
}

#[test]
fn nva_context_expands_lead_and_reputation_triage_contracts() {
    let lead = lead::Lead {
        customer_id: None,
        source: lead::LeadSource::Campaign {
            name: lead::CampaignName::try_new("  holiday boarding  ").unwrap(),
        },
        intent: lead::LeadIntent::BoardingQuote,
        stage: lead::LeadConversionStage::MissingRequirements,
        requested_service: Some(entities::ServiceKind::Boarding),
        next_action: lead::LeadNextAction::RequestVaccineProof,
    };

    assert_eq!(lead.intent, lead::LeadIntent::BoardingQuote);
    match lead.source {
        lead::LeadSource::Campaign { name } => {
            assert_eq!(name.into_inner(), "holiday boarding");
        }
        _ => panic!("expected campaign lead source"),
    }

    let review = reputation::ReputationSignal {
        location_id: entities::LocationId(uuid::Uuid::nil()),
        platform: reputation::ReviewPlatformName::try_new("  Google  ").unwrap(),
        review_id: reputation::ReviewId::try_new("  review-123  ").unwrap(),
        sentiment: reputation::ReviewSentiment::Negative,
        themes: vec![reputation::ReviewTheme::PetInjuryOrSafety],
        escalation: reputation::ReviewEscalation::SafetyOrLegalReviewRequired,
    };

    assert_eq!(review.platform.into_inner(), "Google");
    assert_eq!(review.review_id.into_inner(), "review-123");
    assert_eq!(
        review.escalation,
        reputation::ReviewEscalation::SafetyOrLegalReviewRequired
    );
    assert!(lead::CampaignName::try_new("   ").is_err());
    assert!(reputation::ReviewPlatformName::try_new("   ").is_err());
}

#[test]
fn staff_operations_tasks_encode_due_evidence_and_manager_attention() {
    let task = staff::StaffTask::builder()
        .location_id(entities::LocationId(uuid::Uuid::nil()))
        .kind(staff::StaffTaskKind::MedicationAdministration {
            pet_id: entities::PetId(uuid::Uuid::nil()),
        })
        .title(workflow::task::Title::try_new("  Give evening medication  ").unwrap())
        .status(staff::StaffTaskStatus::Open)
        .priority(staff::StaffTaskPriority::High)
        .due_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .assignment(staff::StaffTaskAssignment::Role(
            staff::StaffRole::KennelTechnician,
        ))
        .source(staff::StaffTaskSource::Reservation(
            entities::ReservationId(uuid::Uuid::nil()),
        ))
        .build();

    assert!(task.requires_manager_attention());
    assert_eq!(task.title.clone().into_inner(), "Give evening medication");

    let completed = task.complete_with(
        staff::TaskCompletionEvidence::try_new(
            "  administered by tech and double-checked by lead  ",
        )
        .unwrap(),
    );

    assert_eq!(completed.status, staff::StaffTaskStatus::Completed);
    assert_eq!(
        completed.completion_evidence.unwrap().into_inner(),
        "administered by tech and double-checked by lead"
    );
    assert!(staff::TaskCompletionEvidence::try_new("   ").is_err());
}

#[test]
fn nva_context_pack_business_services_and_systems_are_typed() {
    let portfolio = operations::PetResortPortfolio::builder()
        .operator(operations::Operator::NationalVeterinaryAssociates)
        .resort_count(operations::ResortCount::try_new(170).unwrap())
        .structure(operations::PortfolioStructure::FederatedMultiBrand)
        .business_lines(vec![operations::BusinessLine::PetResorts])
        .brands(vec![
            operations::PetResortBrand::PetSuites,
            operations::PetResortBrand::PoochHotel,
            operations::PetResortBrand::EliteSuites,
            operations::PetResortBrand::TheBarkSide,
            operations::PetResortBrand::WoofdorfAstoria,
            operations::PetResortBrand::DoggieDistrict,
        ])
        .build();

    assert_eq!(portfolio.resort_count.get(), 170);
    assert!(operations::ResortCount::try_new(0).is_err());
    assert!(
        portfolio
            .brands
            .contains(&operations::PetResortBrand::PetSuites)
    );

    let boarding = operations::ServiceOffering::Boarding {
        accommodation: operations::BoardingAccommodation::LuxurySuite,
        included_care: vec![
            operations::BoardingCareFeature::DailyHousekeeping,
            operations::BoardingCareFeature::PottyWalks,
            operations::BoardingCareFeature::Bedding,
            operations::BoardingCareFeature::PawgressReport,
        ],
        add_ons: vec![
            operations::BoardingAddOn::ExitBath,
            operations::BoardingAddOn::PremiumSuite,
            operations::BoardingAddOn::TrainingSession,
        ],
    };
    let daycare = operations::ServiceOffering::Daycare {
        format: operations::DaycareFormat::AllDayPlay,
        eligibility_rules: vec![operations::DaycareEligibilityRule::SpayNeuterRequiredForGroupPlay],
    };
    let grooming = operations::ServiceOffering::Grooming {
        service: grooming::Service::NailDremel,
        cadence: grooming::RebookingCadence::EveryWeeks(
            grooming::CadenceWeeks::try_new(6).unwrap(),
        ),
    };
    let training = operations::ServiceOffering::Training {
        program: operations::TrainingProgram::StayAndStudy {
            duration: operations::TrainingProgramDurationWeeks::try_new(3).unwrap(),
        },
    };

    assert!(matches!(
        boarding,
        operations::ServiceOffering::Boarding { .. }
    ));
    assert!(matches!(
        daycare,
        operations::ServiceOffering::Daycare { .. }
    ));
    assert!(matches!(
        grooming,
        operations::ServiceOffering::Grooming { .. }
    ));
    assert!(matches!(
        training,
        operations::ServiceOffering::Training { .. }
    ));
    assert!(grooming::CadenceWeeks::try_new(0).is_err());
    assert!(matches!(
        operations::CadenceWeeks::try_new(0),
        Err(operations::CadenceWeeksError::ZeroWeeks)
    ));
    let unknown_cadence = operations::GroomingCadence::Unknown;
    assert!(matches!(
        unknown_cadence,
        grooming::RebookingCadence::Unknown
    ));
    assert!(operations::TrainingProgramDurationWeeks::try_new(0).is_err());

    let ecosystem = operations::TechnologyEcosystem::builder()
        .core_portal(operations::CoreOperatingSystem::Gingr)
        .data_access(vec![
            operations::DataAccessPattern::Api,
            operations::DataAccessPattern::Webhook,
            operations::DataAccessPattern::Warehouse,
        ])
        .adjacent_systems(vec![
            operations::AdjacentSystem::LaborScheduling,
            operations::AdjacentSystem::Reviews,
            operations::AdjacentSystem::EmailSmsMarketing,
            operations::AdjacentSystem::BusinessIntelligence,
        ])
        .build();

    assert_eq!(
        ecosystem.core_portal,
        operations::CoreOperatingSystem::Gingr
    );
    assert!(
        ecosystem
            .adjacent_systems
            .contains(&operations::AdjacentSystem::Reviews)
    );
}

#[test]
fn nva_context_pack_operational_workflows_are_typed() {
    let use_cases = vec![
        operations::AiUseCase::ResortManagerDailyBriefing,
        operations::AiUseCase::RegionalOpsExceptionReporting,
        operations::AiUseCase::CustomerInboxAndCallDeflection,
        operations::AiUseCase::LeadConversion,
        operations::AiUseCase::GroomingRebooking,
        operations::AiUseCase::PostStayPawgressReportAssistant,
        operations::AiUseCase::ReviewReputationTriage,
        operations::AiUseCase::SopKnowledgeAssistant,
        operations::AiUseCase::DataQualityOpsHygiene,
        operations::AiUseCase::IncidentReportDrafting,
        operations::AiUseCase::LapsedCustomerWinback,
        operations::AiUseCase::BoardingPreArrivalChecklistAutomation,
        operations::AiUseCase::CapacityAlerts,
        operations::AiUseCase::LaborRevenueAnomalyDetection,
    ];

    assert!(use_cases.contains(&operations::AiUseCase::LeadConversion));
    assert!(use_cases.contains(&operations::AiUseCase::GroomingRebooking));
    assert!(use_cases.contains(&operations::AiUseCase::DataQualityOpsHygiene));
}

#[test]
fn nva_context_pack_operating_vocabulary_and_data_hygiene_are_typed() {
    let resort_terms = [
        operations::PetResortOperatingTerm::PawgressReports,
        operations::PetResortOperatingTerm::PetPointsRewards,
        operations::PetResortOperatingTerm::LeadCaptureAndConversion,
        operations::PetResortOperatingTerm::ResortLevelEbitdaProfitability,
        operations::PetResortOperatingTerm::DaycareEligibilityRules,
        operations::PetResortOperatingTerm::GuestExperience,
    ];
    assert!(resort_terms.contains(&operations::PetResortOperatingTerm::PawgressReports));

    let hygiene = [
        operations::DataQualityIssue::MissingPetVaccinationRecords,
        operations::DataQualityIssue::DuplicateCustomers,
        operations::DataQualityIssue::UnusedPackages,
        operations::DataQualityIssue::StaffNotesTooVague,
    ];
    assert!(hygiene.contains(&operations::DataQualityIssue::DuplicateCustomers));
}

#[test]
fn entity_builders_keep_optional_and_collection_defaults_semantic() {
    let customer = entities::Customer::builder()
        .id(entities::CustomerId(uuid::Uuid::nil()))
        .full_name(customer::Name::try_new("  Ana Rivera  ").unwrap())
        .preferred_contact(entities::ContactChannel::Email)
        .build();

    assert_eq!(customer.full_name.into_inner(), "Ana Rivera");
    assert_eq!(customer.email, None);
    assert_eq!(customer.mobile_phone, None);
    assert_eq!(customer.portal_account, None);

    let reservation = entities::Reservation::builder()
        .id(entities::ReservationId(uuid::Uuid::nil()))
        .location_id(entities::LocationId(uuid::Uuid::nil()))
        .customer_id(entities::CustomerId(uuid::Uuid::nil()))
        .pet_ids(vec![entities::PetId(uuid::Uuid::nil())])
        .service(entities::ServiceKind::Boarding)
        .status(entities::ReservationStatus::Requested)
        .starts_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .ends_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .source(entities::ReservationSource::WebsiteForm)
        .build();

    assert_eq!(reservation.deposit, None);
    assert!(reservation.requested_add_ons.is_empty());
    assert!(reservation.hard_stops.is_empty());

    let pet = entities::Pet::builder()
        .id(entities::PetId(uuid::Uuid::nil()))
        .customer_id(entities::CustomerId(uuid::Uuid::nil()))
        .name(pet::Name::try_new("  Moose  ").unwrap())
        .species(entities::Species::Dog)
        .spay_neuter_status(entities::SpayNeuterStatus::Neutered)
        .build();

    assert_eq!(pet.name.into_inner(), "Moose");
    assert_eq!(pet.birth_date, None);
    assert_eq!(pet.sex, None);
    assert_eq!(pet.temperament, entities::TemperamentProfile::default());
    assert_eq!(pet.care_profile, entities::CareProfile::default());
}
