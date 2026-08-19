use domain::source;
use strum::VariantArray;

#[test]
fn source_system_external_codes_parse_display_and_iterate_stably() {
    let cases = [
        ("telephony", source::System::Telephony),
        ("sms_provider", source::System::SmsProvider),
        ("email", source::System::Email),
        ("web_chat", source::System::WebChat),
        ("website_forms", source::System::WebsiteForms),
        ("marketing_automation", source::System::MarketingAutomation),
        ("crm", source::System::Crm),
        ("finance_accounting", source::System::FinanceAccounting),
        ("workforce_management", source::System::WorkforceManagement),
        ("knowledge_base", source::System::KnowledgeBase),
        ("provider_or_pms", source::System::ProviderOrPms),
        (
            "business_intelligence",
            source::System::BusinessIntelligence,
        ),
        ("labor_scheduling", source::System::LaborScheduling),
        ("timeclock", source::System::Timeclock),
        ("payroll", source::System::Payroll),
        ("capacity_inventory", source::System::CapacityInventory),
        ("point_of_sale", source::System::PointOfSale),
        ("manual_import", source::System::ManualImport),
    ];

    assert_eq!(source::System::VARIANTS, cases.map(|(_, value)| value));
    for (code, expected) in cases {
        assert_eq!(code.parse::<source::System>().unwrap(), expected);
        assert_eq!(expected.to_string(), code);
    }
}

#[test]
fn source_record_role_external_codes_parse_display_and_iterate_stably() {
    let cases = [
        ("customer", source::record::Role::Customer),
        ("pet", source::record::Role::Pet),
        ("location", source::record::Role::Location),
        ("reservation_type", source::record::Role::ReservationType),
        ("invoice", source::record::Role::Invoice),
        ("payment", source::record::Role::Payment),
        ("service", source::record::Role::Service),
        ("staff", source::record::Role::Staff),
        ("unknown", source::record::Role::Unknown),
    ];

    assert_eq!(
        source::record::Role::VARIANTS,
        cases.map(|(_, value)| value)
    );
    for (code, expected) in cases {
        assert_eq!(code.parse::<source::record::Role>().unwrap(), expected);
        assert_eq!(expected.to_string(), code);
    }
}
