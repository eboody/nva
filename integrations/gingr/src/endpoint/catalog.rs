pub fn exported_read_endpoint_names() -> &'static [&'static str] {
    &[
        "get_locations",
        "get_species",
        "get_breeds",
        "get_vets",
        "get_temperaments",
        "get_immunization_types",
        "get_animal_immunizations",
        "reservation_types",
        "get_services_by_type",
        "reservation_widget_data",
        "reservations",
        "reservations_by_animal",
        "reservations_by_owner",
        "back_of_house",
        "owner",
        "owners",
        "animals",
        "forms_get_form",
        "custom_field_search",
        "get_feeding_info",
        "get_medication_info",
        "get_all_retail_items",
        "list_transactions",
        "transaction",
        "list_invoices",
        "get_subscription",
        "get_subscriptions",
        "timeclock_report",
        "report_card_files",
    ]
}

pub fn semantic_mapping_gaps() -> &'static [&'static str] {
    &["retail", "training", "grooming"]
}
