use domain::{care, customer, source};

fn string_with_chars(count: usize) -> String {
    "x".repeat(count)
}

#[test]
fn customer_contact_values_trim_reject_blank_and_validate_serde() {
    let name = customer::Name::try_new("  Ada Lovelace  ").unwrap();
    let email = customer::Email::try_new("  ada@example.com  ").unwrap();
    let phone = customer::Phone::try_new("  +1 555 0100  ").unwrap();

    assert_eq!(name.into_inner(), "Ada Lovelace");
    assert_eq!(email.into_inner(), "ada@example.com");
    assert_eq!(phone.into_inner(), "+1 555 0100");

    assert!(customer::Name::try_new("   ").is_err());
    assert!(customer::Email::try_new("   ").is_err());
    assert!(customer::Phone::try_new("   ").is_err());
    assert!(customer::Name::try_new(string_with_chars(121)).is_err());
    assert!(customer::Email::try_new(string_with_chars(255)).is_err());
    assert!(customer::Phone::try_new(string_with_chars(41)).is_err());

    let serialized = serde_json::to_string(&customer::Name::try_new("  Grace  ").unwrap()).unwrap();
    assert_eq!(serialized, "\"Grace\"");
    assert!(serde_json::from_str::<customer::Name>("\"   \"").is_err());
}

#[test]
fn care_values_redact_debug_trim_reject_blank_and_validate_serde() {
    let contact = care::ContactName::try_new("  Dr. Smith  ").unwrap();
    let medication = care::MedicationName::try_new("  Insulin  ").unwrap();
    let note = care::MedicalNote::try_new("  Give with food  ").unwrap();

    assert_eq!(contact.clone().into_inner(), "Dr. Smith");
    assert_eq!(medication.clone().into_inner(), "Insulin");
    assert_eq!(note.clone().into_inner(), "Give with food");

    assert_eq!(format!("{contact:?}"), "ContactName(<redacted>)");
    assert_eq!(format!("{medication:?}"), "MedicationName(<redacted>)");
    assert_eq!(format!("{note:?}"), "MedicalNote(<redacted>)");
    assert!(!format!("{note:?}").contains("Give with food"));

    assert!(care::ContactName::try_new("   ").is_err());
    assert!(care::MedicationName::try_new("   ").is_err());
    assert!(care::MedicalNote::try_new("   ").is_err());
    assert!(care::ContactName::try_new(string_with_chars(161)).is_err());
    assert!(care::MedicationName::try_new(string_with_chars(161)).is_err());
    assert!(care::MedicalNote::try_new(string_with_chars(1001)).is_err());

    let serialized =
        serde_json::to_string(&care::ContactName::try_new("  Care lead  ").unwrap()).unwrap();
    assert_eq!(serialized, "\"Care lead\"");
    assert!(serde_json::from_str::<care::ContactName>("\"   \"").is_err());
}

#[test]
fn source_record_id_is_nutype_validated_and_deserialize_cannot_bypass_constructor() {
    let record_id = source::record::Id::try_new("  reservation-123  ").unwrap();

    assert_eq!(record_id.as_str(), "reservation-123");
    assert!(source::record::Id::try_new("   ").is_err());
    assert!(source::record::Id::try_new(string_with_chars(121)).is_err());

    let serialized = serde_json::to_string(&record_id).unwrap();
    assert_eq!(serialized, "\"reservation-123\"");
    assert!(serde_json::from_str::<source::record::Id>("\"   \"").is_err());
    assert!(
        serde_json::from_str::<source::record::Id>(&format!("\"{}\"", string_with_chars(121)))
            .is_err()
    );
}
