#[test]
fn core_service_contract_records_roundtrip_between_storage_and_domain() {
    let domain_contracts = domain::operations::service_core::ServiceContracts::builder()
        .location_id(domain::entities::LocationId(uuid::Uuid::nil()))
        .boarding(domain::boarding::Contract::standard_petsuites())
        .daycare(domain::daycare::Contract::standard_petsuites())
        .grooming(domain::grooming::Contract::standard_petsuites())
        .training(domain::training::Contract::standard_petsuites())
        .retail(domain::retail::Contract::standard_petsuites())
        .build();

    let record: storage::operations::CoreServiceContractsRecord = domain_contracts.clone().into();
    let encoded = record.encode_json().unwrap();
    let decoded_record =
        storage::operations::CoreServiceContractsRecord::decode_json(&encoded).unwrap();
    let decoded_domain: domain::operations::service_core::ServiceContracts = decoded_record.into();

    assert_eq!(decoded_domain, domain_contracts);
    assert!(decoded_domain.boarding.requires_deposit_collection());
    assert!(
        decoded_domain
            .daycare
            .requires_staff_review_before_group_play()
    );
    assert!(decoded_domain.retail.should_reorder());
}

#[test]
fn core_service_contract_codecs_reject_invalid_validated_scalars() {
    let raw = r#"
    {
      "location_id": "00000000-0000-0000-0000-000000000000",
      "boarding": {
        "capacity": { "room_inventory": 0, "availability": "Limited" },
        "arrival_window": { "start": 7, "end": 18 },
        "departure_window": { "start": 7, "end": 12 },
        "minimum_stay": { "nights": 1, "reason": "StandardPolicy" },
        "cancellation": { "notice": 24, "penalty": "ForfeitDeposit" },
        "deposit": { "Required": { "amount": { "minor_units": 1, "currency": "Usd" } } },
        "payment": "DueAtCheckout",
        "housekeeping": "DailyRoomReset",
        "handoff": "ArrivalCareReview",
        "upsells": ["ExitBath"]
      },
      "daycare": {
        "attendance": "ReservationRequired",
        "package": { "PrepaidPasses": { "visits": 5 } },
        "ratio": { "staff": 1, "pets": 12 },
        "group_assignment": "TemperamentAndSizeMatched",
        "incident": "ManagerReviewAndCustomerNotice",
        "eligibility": ["TemperamentAssessment"]
      },
      "grooming": {
        "calendar": "GroomerSpecific",
        "time_estimates": [],
        "no_show": "RequireDepositForRebooking",
        "rebooking": { "EveryWeeks": 6 },
        "reminders": ["MorningOf"],
        "history": "KeepStyleNotesAndPhotos"
      },
      "training": {
        "program_duration": { "Weeks": 3 },
        "curriculum": ["Recall"],
        "progress": "SessionNotesAndMilestones",
        "outcomes": ["CanineGoodCitizenReadiness"],
        "trainer_availability": "NamedTrainerRequired",
        "package": { "MultiSessionPackage": { "sessions": 6 } },
        "follow_up": "AfterProgramCompletion"
      },
      "retail": {
        "product": { "sku": "PETSUITES-RETAIL", "category": "PersonalizedUpsell" },
        "pos": "IntegratedWithReservationCheckout",
        "inventory": { "Tracked": { "on_hand": 1, "reorder_at": 10 } },
        "recommendation": "AnxietySupportAfterBoarding",
        "reorder": "AutoCreateManagerTask"
      }
    }
    "#;

    let err = storage::operations::CoreServiceContractsRecord::decode_json(raw).unwrap_err();

    assert!(matches!(
        err,
        storage::operations::Error::Codec(storage::operations::CodecError::JsonDecode {
            record: storage::operations::RecordKind::CoreServiceContracts,
            ..
        })
    ));
    assert!(err.to_string().contains("CoreServiceContracts"));
}
