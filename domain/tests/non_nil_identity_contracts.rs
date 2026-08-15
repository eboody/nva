use domain::entities;

#[test]
fn production_domain_identities_reject_nil_at_constructor_and_serde_boundaries() {
    let nil = uuid::Uuid::nil();

    assert!(entities::LocationId::try_new(nil).is_err());
    assert!(entities::CustomerId::try_new(nil).is_err());
    assert!(entities::PetId::try_new(nil).is_err());
    assert!(entities::IncidentId::try_new(nil).is_err());
    assert!(entities::MessageId::try_new(nil).is_err());
    assert!(entities::reservation::Id::try_new(nil).is_err());
    assert!(entities::approval::Id::try_new(nil).is_err());
    assert!(entities::care_note::Id::try_new(nil).is_err());
    assert!(entities::DocumentId::try_new(nil).is_err());
    assert!(entities::VaccineRecordId::try_new(nil).is_err());
    assert!(domain::workflow::EventId::try_new(nil).is_err());
    assert!(domain::audit::EventId::try_new(nil).is_err());

    let encoded_nil = serde_json::to_value(nil).unwrap();
    assert!(serde_json::from_value::<entities::LocationId>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<entities::CustomerId>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<entities::PetId>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<entities::IncidentId>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<entities::MessageId>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<entities::reservation::Id>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<entities::approval::Id>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<entities::care_note::Id>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<entities::DocumentId>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<entities::VaccineRecordId>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<domain::workflow::EventId>(encoded_nil.clone()).is_err());
    assert!(serde_json::from_value::<domain::audit::EventId>(encoded_nil).is_err());
}
