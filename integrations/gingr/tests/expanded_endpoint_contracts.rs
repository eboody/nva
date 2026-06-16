use domain::retail;
use gingr::{config, dto, endpoint, mapping, response, transport};

const SENTINEL_KEY: &str = "gingr_test_api_key_do_not_send";

fn fake_client() -> transport::Client<transport::MockTransport> {
    let config = config::Client::new(
        config::BaseUrl::parse("https://example-pet-resort.gingrapp.com").unwrap(),
        config::ApiKey::from_secret(SENTINEL_KEY),
    );
    transport::Client::with_transport(config, transport::MockTransport)
}

#[test]
fn reservations_range_post_uses_form_body_and_rejects_ranges_longer_than_30_days() {
    let client = fake_client();
    let range = endpoint::DateRange::new(
        endpoint::Date::parse("2026-06-01").unwrap(),
        endpoint::Date::parse("2026-06-30").unwrap(),
    )
    .unwrap();
    let request = endpoint::reservations::Reservations::for_range(range)
        .location(endpoint::LocationId::new(7))
        .build();

    let sent = client.capture_request(&request).unwrap();
    let redacted = client.redacted_request(&request).unwrap().to_string();

    assert_eq!(sent.method(), endpoint::Method::Post);
    assert_eq!(sent.path(), "/api/v1/reservations");
    assert!(sent.query_pairs().is_empty());
    assert!(
        sent.form_pairs()
            .contains(&("checked_in".into(), "false".into()))
    );
    assert!(
        sent.form_pairs()
            .contains(&("start_date".into(), "2026-06-01".into()))
    );
    assert!(
        sent.form_pairs()
            .contains(&("end_date".into(), "2026-06-30".into()))
    );
    assert!(
        sent.form_pairs()
            .contains(&("location_id".into(), "7".into()))
    );
    assert!(
        sent.form_pairs()
            .contains(&("key".into(), SENTINEL_KEY.into()))
    );
    assert!(!redacted.contains(SENTINEL_KEY));
    assert!(redacted.contains("form:"));
    assert!(redacted.contains("key=<redacted>"));

    assert!(
        endpoint::DateRange::new(
            endpoint::Date::parse("2026-06-01").unwrap(),
            endpoint::Date::parse("2026-07-02").unwrap(),
        )
        .is_err()
    );
}

#[test]
fn reservations_by_pet_and_owner_encode_restrict_to_and_location_scope_caveat_filters() {
    let client = fake_client();
    let animal_request = endpoint::reservations::by::Animal::builder()
        .animal_id(endpoint::AnimalId::new(23))
        .restrict_to(endpoint::reservations::RestrictTo::Future)
        .filter(
            endpoint::reservations::reservation::SearchFilters::builder()
                .from_date(endpoint::IsoDate::parse("2026-06-01").unwrap())
                .to_date(endpoint::IsoDate::parse("2026-06-30").unwrap())
                .reservation_type_id(endpoint::reservations::reservation::TypeId::new(2))
                .animal_id(endpoint::AnimalId::new(23))
                .confirmed_only(true)
                .limit(endpoint::Limit::new(50).unwrap())
                .build(),
        )
        .build();
    let owner_request = endpoint::reservations::by::Owner::builder()
        .owner_id(endpoint::OwnerId::new(99))
        .restrict_to(endpoint::reservations::RestrictTo::CurrentlyCheckedIn)
        .build();

    let animal_sent = client.capture_request(&animal_request).unwrap();
    let owner_sent = client.capture_request(&owner_request).unwrap();

    assert_eq!(animal_sent.path(), "/api/v1/reservations_by_animal");
    assert!(
        animal_sent
            .form_pairs()
            .contains(&("id".into(), "23".into()))
    );
    assert!(
        animal_sent
            .form_pairs()
            .contains(&("restrict_to".into(), "future".into()))
    );
    assert!(
        animal_sent
            .form_pairs()
            .contains(&("params[fromDate]".into(), "2026-06-01".into()))
    );
    assert!(
        animal_sent
            .form_pairs()
            .contains(&("params[toDate]".into(), "2026-06-30".into()))
    );
    assert!(
        animal_sent
            .form_pairs()
            .contains(&("params[reservationTypeIds][]".into(), "2".into()))
    );
    assert!(
        animal_sent
            .form_pairs()
            .contains(&("params[animalIds][]".into(), "23".into()))
    );
    assert!(
        animal_sent
            .form_pairs()
            .contains(&("params[confirmedOnly]".into(), "true".into()))
    );
    assert!(
        animal_sent
            .form_pairs()
            .contains(&("params[limit]".into(), "50".into()))
    );
    assert_eq!(owner_sent.path(), "/api/v1/reservations_by_owner");
    assert!(
        owner_sent
            .form_pairs()
            .contains(&("id".into(), "99".into()))
    );
    assert!(
        owner_sent
            .form_pairs()
            .contains(&("restrict_to".into(), "currently_checked_in".into()))
    );
    assert!(
        endpoint::reservations::by::Animal::LOCATION_SCOPE_CAVEAT.contains("currently logged into")
    );
}

#[test]
fn owner_lookup_requires_one_discriminator_and_reference_endpoints_stay_typed() {
    let client = fake_client();
    let owner = endpoint::owners_animals::Owner::by_email(
        endpoint::owners_animals::SensitiveLookup::new("ana@example.test").unwrap(),
    );
    let immunizations =
        endpoint::reference_data::GetImmunizationTypes::new(endpoint::SpeciesId::new(1));
    let vets = endpoint::reference_data::GetVets::builder()
        .include_all_information(true)
        .build();

    let owner_sent = client.capture_request(&owner).unwrap();
    let immunizations_sent = client.capture_request(&immunizations).unwrap();
    let vets_sent = client.capture_request(&vets).unwrap();

    assert_eq!(owner_sent.path(), "/api/v1/owner");
    assert!(
        owner_sent
            .query_pairs()
            .contains(&("email".into(), "ana@example.test".into()))
    );
    assert_eq!(immunizations_sent.path(), "/api/v1/get_immunization_types");
    assert!(
        immunizations_sent
            .query_pairs()
            .contains(&("species_id".into(), "1".into()))
    );
    assert_eq!(vets_sent.path(), "/api/v1/get_vets");
    assert!(
        vets_sent
            .query_pairs()
            .contains(&("vetFlag".into(), "true".into()))
    );
}

#[test]
fn forms_custom_field_and_back_of_house_are_explicitly_sensitive_or_v0_safe() {
    let client = fake_client();
    let form = endpoint::owners_animals::Form::new(endpoint::owners_animals::FormKind::Animal);
    let custom_search = endpoint::owners_animals::custom_field::Search::builder()
        .form(endpoint::owners_animals::FormKind::Owner)
        .field_name(endpoint::owners_animals::custom_field::Name::new("preferred_contact").unwrap())
        .search(endpoint::owners_animals::SensitiveLookup::new("sms").unwrap())
        .build();
    let whiteboard = endpoint::reservations::BackOfHouse::builder()
        .location(endpoint::LocationId::new(3))
        .reservation_type_id(endpoint::reservations::reservation::TypeId::new(4))
        .minutes_future(endpoint::reservations::MinutesFuture::new(120).unwrap())
        .build();

    let form_sent = client.capture_request(&form).unwrap();
    let custom_sent = client.capture_request(&custom_search).unwrap();
    let whiteboard_sent = client.capture_request(&whiteboard).unwrap();
    let redacted_custom = client.redacted_request(&custom_search).unwrap().to_string();

    assert_eq!(form_sent.path(), "/forms/get_form");
    assert!(
        form_sent
            .query_pairs()
            .contains(&("form".into(), "animal_form".into()))
    );
    assert_eq!(custom_sent.path(), "/api/v1/custom_field_search");
    assert!(
        custom_sent
            .query_pairs()
            .contains(&("form_id".into(), "1".into()))
    );
    assert!(
        custom_sent
            .query_pairs()
            .contains(&("field_name".into(), "preferred_contact".into()))
    );
    assert!(
        custom_sent
            .query_pairs()
            .contains(&("search".into(), "sms".into()))
    );
    assert!(!redacted_custom.contains("sms"));
    assert!(redacted_custom.contains("search=<redacted>"));
    assert_eq!(whiteboard_sent.path(), "/api/v1/back_of_house");
    assert!(
        whiteboard_sent
            .query_pairs()
            .contains(&("location_id".into(), "3".into()))
    );
    assert!(
        whiteboard_sent
            .query_pairs()
            .contains(&("type_ids[]".into(), "4".into()))
    );
    assert!(
        whiteboard_sent
            .query_pairs()
            .contains(&("mins_future".into(), "120".into()))
    );
}

#[test]
fn provider_dtos_preserve_unknown_fields_and_mappers_promote_only_existing_domain_values() {
    let owner: response::OwnerRecord = serde_json::from_value(serde_json::json!({
        "id": 42,
        "first_name": " Ana ",
        "last_name": " Rivera ",
        "email": " ana@example.test ",
        "cell_phone": " +1 555 0100 ",
        "password": "provider-secret-shape-is-quarantined"
    }))
    .unwrap();
    let animal: response::AnimalRecord = serde_json::from_value(serde_json::json!({
        "id": 9,
        "owner_id": 42,
        "name": " Juniper ",
        "species": "Dog",
        "birthday": "2021-04-03",
        "custom_provider_blob": {"ignored_by_mapping": true}
    }))
    .unwrap();

    let customer = mapping::customer::contact_candidate(&owner).unwrap();
    let pet = mapping::pet::name_candidate(&animal).unwrap();

    assert_eq!(customer.full_name.into_inner(), "Ana Rivera");
    assert_eq!(customer.email.unwrap().into_inner(), "ana@example.test");
    assert_eq!(customer.mobile_phone.unwrap().into_inner(), "+1 555 0100");
    assert_eq!(owner.id, endpoint::OwnerId::new(42));
    assert_eq!(owner.email.as_ref().unwrap().as_str(), " ana@example.test ");
    assert_eq!(customer.provider_owner_id, endpoint::OwnerId::new(42));
    assert_eq!(pet.name.into_inner(), "Juniper");
    assert_eq!(animal.id, endpoint::AnimalId::new(9));
    assert_eq!(animal.owner_id, Some(endpoint::OwnerId::new(42)));
    assert_eq!(pet.provider_animal_id, endpoint::AnimalId::new(9));
    assert!(owner.unknown.contains_key("password"));
    assert!(animal.unknown.contains_key("custom_provider_blob"));
}

#[test]
fn retail_item_dto_promotes_documented_provider_surface_into_retail_product_candidate() {
    let item: dto::retail::Item = serde_json::from_value(serde_json::json!({
        "id": 41,
        "name": " Calming Chew ",
        "sku": " CALM-CHEW ",
        "category": "supplement",
        "active": true,
        "quantity_on_hand": 7,
        "provider_only_shape": {"kept": true}
    }))
    .unwrap();

    let candidate = mapping::retail::product_candidate(&item).unwrap();

    assert_eq!(candidate.provider_item_id, dto::retail::ItemId::new(41));
    assert_eq!(candidate.name.into_inner(), "Calming Chew");
    assert_eq!(candidate.product.sku().as_str(), "CALM-CHEW");
    assert_eq!(
        candidate.product.category,
        retail::product::Category::Supplement
    );
    assert_eq!(candidate.status, retail::OfferingStatus::Active);
    assert!(item.unknown.contains_key("provider_only_shape"));
}

#[test]
fn grooming_and_training_surfaces_remain_explicit_provider_gaps_without_fake_dtos() {
    assert_eq!(
        dto::grooming::provider_surface(),
        dto::ProviderSurface::NoDocumentedServiceDto {
            endpoint: "get_services_by_type"
        }
    );
    assert_eq!(
        dto::training::provider_surface(),
        dto::ProviderSurface::NoDocumentedServiceDto {
            endpoint: "get_services_by_type"
        }
    );
    assert!(endpoint::catalog::semantic_mapping_gaps().contains(&"grooming"));
    assert!(endpoint::catalog::semantic_mapping_gaps().contains(&"training"));
}
