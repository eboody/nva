use storage::operations::{
    DataQualityHygieneActionKindCode, DataQualityHygieneOutcomeCode, DataQualityHygienePersonaCode,
    DataQualityResolutionStatusCode, OperatorCode, PetResortBrandCode,
};
use strum::VariantArray;

#[test]
fn portfolio_storage_codes_parse_display_and_iterate_stably() {
    assert_eq!(
        "nva".parse::<OperatorCode>().unwrap(),
        OperatorCode::NationalVeterinaryAssociates
    );
    assert_eq!(
        OperatorCode::NationalVeterinaryAssociates.to_string(),
        "nva"
    );

    let brand_cases = [
        ("nva_pet_resorts", PetResortBrandCode::NvaPetResorts),
        ("pet_suites", PetResortBrandCode::PetSuites),
        ("pooch_hotel", PetResortBrandCode::PoochHotel),
        ("elite_suites", PetResortBrandCode::EliteSuites),
        ("the_bark_side", PetResortBrandCode::TheBarkSide),
        ("woofdorf_astoria", PetResortBrandCode::WoofdorfAstoria),
        ("doggie_district", PetResortBrandCode::DoggieDistrict),
    ];

    assert_eq!(
        PetResortBrandCode::VARIANTS,
        brand_cases.map(|(_, value)| value)
    );
    for (code, expected) in brand_cases {
        assert_eq!(code.parse::<PetResortBrandCode>().unwrap(), expected);
        assert_eq!(expected.to_string(), code);
    }
}

#[test]
fn data_quality_hygiene_storage_codes_parse_display_and_iterate_stably() {
    let outcome_cases = [
        ("completed", DataQualityHygieneOutcomeCode::Completed),
        ("deferred", DataQualityHygieneOutcomeCode::Deferred),
        (
            "suppressed_by_manager",
            DataQualityHygieneOutcomeCode::SuppressedByManager,
        ),
        (
            "source_fact_was_wrong",
            DataQualityHygieneOutcomeCode::SourceFactWasWrong,
        ),
        (
            "not_actionable",
            DataQualityHygieneOutcomeCode::NotActionable,
        ),
    ];
    assert_eq!(
        DataQualityHygieneOutcomeCode::VARIANTS,
        outcome_cases.map(|(_, value)| value)
    );
    for (code, expected) in outcome_cases {
        assert_eq!(
            code.parse::<DataQualityHygieneOutcomeCode>().unwrap(),
            expected
        );
        assert_eq!(expected.to_string(), code);
    }

    let persona_cases = [
        (
            "general_manager",
            DataQualityHygienePersonaCode::GeneralManager,
        ),
        (
            "assistant_general_manager",
            DataQualityHygienePersonaCode::AssistantGeneralManager,
        ),
        (
            "front_desk_lead",
            DataQualityHygienePersonaCode::FrontDeskLead,
        ),
        (
            "front_desk_agent",
            DataQualityHygienePersonaCode::FrontDeskAgent,
        ),
        (
            "regional_operator",
            DataQualityHygienePersonaCode::RegionalOperator,
        ),
        (
            "operations_analyst",
            DataQualityHygienePersonaCode::OperationsAnalyst,
        ),
    ];
    assert_eq!(
        DataQualityHygienePersonaCode::VARIANTS,
        persona_cases.map(|(_, value)| value)
    );
    for (code, expected) in persona_cases {
        assert_eq!(
            code.parse::<DataQualityHygienePersonaCode>().unwrap(),
            expected
        );
        assert_eq!(expected.to_string(), code);
    }

    let action_cases = [
        (
            "investigate_missing_source_evidence",
            DataQualityHygieneActionKindCode::InvestigateMissingSourceEvidence,
        ),
        (
            "reconcile_duplicate_customer_or_pet_candidate",
            DataQualityHygieneActionKindCode::ReconcileDuplicateCustomerOrPetCandidate,
        ),
        (
            "complete_missing_pet_or_customer_profile_fields",
            DataQualityHygieneActionKindCode::CompleteMissingPetOrCustomerProfileFields,
        ),
        (
            "review_stale_vaccination_source_freshness",
            DataQualityHygieneActionKindCode::ReviewStaleVaccinationSourceFreshness,
        ),
        (
            "normalize_ambiguous_service_line_naming",
            DataQualityHygieneActionKindCode::NormalizeAmbiguousServiceLineNaming,
        ),
        (
            "review_checkout_or_unclosed_reservation_evidence",
            DataQualityHygieneActionKindCode::ReviewCheckoutOrUnclosedReservationEvidence,
        ),
        (
            "escalate_sensitive_or_quarantined_payload",
            DataQualityHygieneActionKindCode::EscalateSensitiveOrQuarantinedPayload,
        ),
        (
            "review_payment_state_conflict",
            DataQualityHygieneActionKindCode::ReviewPaymentStateConflict,
        ),
    ];
    assert_eq!(
        DataQualityHygieneActionKindCode::VARIANTS,
        action_cases.map(|(_, value)| value)
    );
    for (code, expected) in action_cases {
        assert_eq!(
            code.parse::<DataQualityHygieneActionKindCode>().unwrap(),
            expected
        );
        assert_eq!(expected.to_string(), code);
    }

    let resolution_cases = [
        ("open", DataQualityResolutionStatusCode::Open),
        (
            "acknowledged",
            DataQualityResolutionStatusCode::Acknowledged,
        ),
        ("ignored", DataQualityResolutionStatusCode::Ignored),
        ("repaired", DataQualityResolutionStatusCode::Repaired),
        ("superseded", DataQualityResolutionStatusCode::Superseded),
    ];
    assert_eq!(
        DataQualityResolutionStatusCode::VARIANTS,
        resolution_cases.map(|(_, value)| value)
    );
    for (code, expected) in resolution_cases {
        assert_eq!(
            code.parse::<DataQualityResolutionStatusCode>().unwrap(),
            expected
        );
        assert_eq!(expected.to_string(), code);
    }
}
