#[test]
fn portfolio_records_promote_into_domain_portfolios_and_demote_back_to_storage() {
    let record = storage::operations::PetResortPortfolioRecord::builder()
        .operator(storage::operations::OperatorCode::NationalVeterinaryAssociates)
        .resort_count(storage::operations::StoredResortCount::try_new(170).unwrap())
        .structure(storage::operations::PortfolioStructureCode::FederatedMultiBrand)
        .business_lines(vec![
            storage::operations::BusinessLineCode::PetResorts,
            storage::operations::BusinessLineCode::GeneralPracticeVeterinaryHospitals,
        ])
        .brands(vec![
            storage::operations::PetResortBrandRecord::Known {
                code: storage::operations::PetResortBrandCode::PetSuites,
            },
            storage::operations::PetResortBrandRecord::Known {
                code: storage::operations::PetResortBrandCode::DoggieDistrict,
            },
        ])
        .build();

    let domain_portfolio: domain::operations::PetResortPortfolio =
        record.clone().try_into().unwrap();

    assert_eq!(domain_portfolio.resort_count.get(), 170);
    assert!(
        domain_portfolio
            .business_lines
            .contains(&domain::operations::BusinessLine::PetResorts)
    );
    assert!(
        domain_portfolio
            .brands
            .contains(&domain::operations::PetResortBrand::PetSuites)
    );

    let stored_again: storage::operations::PetResortPortfolioRecord =
        domain_portfolio.try_into().unwrap();
    assert_eq!(stored_again, record);
}

#[test]
fn service_offering_records_preserve_variant_contracts_through_codecs() {
    let grooming = domain::operations::ServiceOffering::Grooming {
        service: domain::grooming::Service::FullGroom,
        cadence: domain::grooming::rebooking::Cadence::EveryWeeks(
            domain::grooming::rebooking::CadenceWeeks::try_new(6).unwrap(),
        ),
    };

    let record: storage::operations::ServiceOfferingRecord = grooming.clone().try_into().unwrap();

    assert_eq!(
        record.service_kind,
        storage::operations::ServiceOfferingKindCode::Grooming
    );
    assert_eq!(
        record.grooming_service,
        Some(storage::service::grooming::ServiceCode::FullGroom)
    );
    assert_eq!(record.grooming_cadence_weeks.unwrap().get(), 6);
    assert!(record.boarding_accommodation.is_none());
    assert!(record.daycare_format.is_none());

    let decoded: domain::operations::ServiceOffering = record.try_into().unwrap();
    assert_eq!(decoded, grooming);
}

#[test]
fn service_offering_records_reject_cross_variant_storage_shapes() {
    let invalid = storage::operations::ServiceOfferingRecord::builder()
        .service_kind(storage::operations::ServiceOfferingKindCode::Grooming)
        .boarding_accommodation(storage::service::boarding::AccommodationCode::LuxurySuite)
        .build();

    let err = domain::operations::ServiceOffering::try_from(invalid).unwrap_err();

    assert!(matches!(
        err,
        storage::operations::Error::StorageShapeMismatch {
            record: storage::operations::RecordKind::ServiceOffering,
            ..
        }
    ));
}

#[test]
fn operations_reexports_narrow_legacy_storage_compatibility_names() {
    let _: storage::operations::StoredCadenceWeeksError =
        storage::service::grooming::StoredCadenceWeeksError::ZeroWeeks;
    let _: storage::operations::StoredTrainingProgramDurationWeeksError =
        storage::service::training::StoredProgramDurationWeeksError::ZeroWeeks;
    let duration: storage::operations::StoredTrainingProgramDurationWeeks =
        storage::service::training::StoredProgramDurationWeeks::try_new(4).unwrap();

    assert_eq!(duration.get(), 4);
}

#[test]
fn service_line_records_promote_domain_service_values_at_storage_boundary() {
    let training_record = storage::service::training::ProgramRecord::try_from(
        domain::training::Program::StayAndStudy {
            duration: domain::training::program::DurationWeeks::try_new(4).unwrap(),
        },
    )
    .unwrap();

    assert_eq!(
        training_record,
        storage::service::training::ProgramRecord::StayAndStudy {
            duration_weeks: storage::service::training::StoredProgramDurationWeeks::try_new(4)
                .unwrap(),
        }
    );

    let domain_program: domain::training::Program = training_record.try_into().unwrap();
    assert_eq!(
        domain_program,
        domain::training::Program::StayAndStudy {
            duration: domain::training::program::DurationWeeks::try_new(4).unwrap(),
        }
    );

    let retail_partner: domain::retail::Partner =
        storage::service::retail::PartnerCode::PurinaEnBoardingDiet.into();
    assert_eq!(
        storage::service::retail::PartnerCode::from(retail_partner),
        storage::service::retail::PartnerCode::PurinaEnBoardingDiet
    );
}

#[test]
fn technology_ecosystem_records_roundtrip_between_storage_and_domain() {
    let domain_ecosystem = domain::operations::TechnologyEcosystem::builder()
        .core_portal(domain::operations::CoreOperatingSystem::Gingr)
        .data_access(vec![
            domain::operations::DataAccessPattern::Api,
            domain::operations::DataAccessPattern::Webhook,
            domain::operations::DataAccessPattern::Warehouse,
        ])
        .adjacent_systems(vec![
            domain::operations::AdjacentSystem::Reviews,
            domain::operations::AdjacentSystem::LaborScheduling,
            domain::operations::AdjacentSystem::EmailSmsMarketing,
        ])
        .build();

    let record: storage::operations::TechnologyEcosystemRecord = domain_ecosystem.clone().into();

    assert_eq!(
        record.core_portal,
        storage::operations::CoreOperatingSystemCode::Gingr
    );
    assert!(
        record
            .data_access
            .contains(&storage::operations::DataAccessPatternCode::Webhook)
    );
    assert!(
        record
            .adjacent_systems
            .contains(&storage::operations::AdjacentSystemCode::Reviews)
    );

    let decoded: domain::operations::TechnologyEcosystem = record.into();
    assert_eq!(decoded, domain_ecosystem);
}

#[test]
fn storage_json_codecs_validate_semantic_newtypes_at_the_boundary() {
    let raw = r#"
    {
      "operator": "nva",
      "resort_count": 0,
      "structure": "federated_multi_brand",
      "business_lines": ["pet_resorts"],
      "brands": [{"kind":"known","code":"pet_suites"}]
    }
    "#;

    let err = storage::operations::PetResortPortfolioRecord::decode_json(raw).unwrap_err();

    assert!(matches!(
        err,
        storage::operations::Error::Codec(storage::operations::CodecError::JsonDecode { .. })
    ));
}
