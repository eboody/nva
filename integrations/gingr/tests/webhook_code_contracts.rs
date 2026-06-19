use gingr::webhook;

#[test]
fn webhook_event_type_provider_codes_parse_and_display_byte_for_byte() {
    let cases = [
        ("check_in", webhook::EventType::CheckIn),
        ("check_out", webhook::EventType::CheckOut),
        ("checking_in", webhook::EventType::CheckingIn),
        ("checking_out", webhook::EventType::CheckingOut),
        ("email_sent", webhook::EventType::EmailSent),
        ("owner_created", webhook::EventType::OwnerCreated),
        ("owner_edited", webhook::EventType::OwnerEdited),
        ("animal_created", webhook::EventType::AnimalCreated),
        ("animal_edited", webhook::EventType::AnimalEdited),
        ("incident_created", webhook::EventType::IncidentCreated),
        ("incident_edited", webhook::EventType::IncidentEdited),
        ("lead_created", webhook::EventType::LeadCreated),
    ];

    for (code, expected) in cases {
        assert_eq!(code.parse::<webhook::EventType>().unwrap(), expected);
        assert_eq!(expected.to_string(), code);
        assert_eq!(expected.as_provider_str(), code);
    }

    let unknown = "boarding_photo_uploaded"
        .parse::<webhook::EventType>()
        .unwrap();
    assert_eq!(
        unknown,
        webhook::EventType::Unknown("boarding_photo_uploaded".to_owned())
    );
    assert_eq!(unknown.to_string(), "boarding_photo_uploaded");
    assert_eq!(unknown.as_provider_str(), "boarding_photo_uploaded");
}

#[test]
fn webhook_entity_type_provider_codes_parse_and_display_byte_for_byte() {
    let cases = [
        ("reservation", webhook::EntityType::Reservation),
        ("owner", webhook::EntityType::Owner),
        ("animal", webhook::EntityType::Animal),
        ("incident", webhook::EntityType::Incident),
        ("lead", webhook::EntityType::Lead),
    ];

    for (code, expected) in cases {
        assert_eq!(code.parse::<webhook::EntityType>().unwrap(), expected);
        assert_eq!(expected.to_string(), code);
        assert_eq!(expected.as_provider_str(), code);
    }

    let unknown = "invoice".parse::<webhook::EntityType>().unwrap();
    assert_eq!(unknown, webhook::EntityType::Unknown("invoice".to_owned()));
    assert_eq!(unknown.to_string(), "invoice");
    assert_eq!(unknown.as_provider_str(), "invoice");
}
