use std::collections::BTreeMap;

use domain::source;
use gingr::{dto, endpoint, mapping, response};

#[test]
fn customer_pet_and_retail_promotions_require_source_backed_mapping_version_before_domain_use() {
    let owner = response::OwnerRecord {
        id: endpoint::OwnerId::new(501),
        first_name: Some("Sam".to_owned()),
        last_name: Some("Rivera".to_owned()),
        email: Some(response::provider::Email::new("sam@example.test")),
        cell_phone: None,
        unknown: BTreeMap::new(),
    };
    let animal = response::AnimalRecord {
        id: endpoint::AnimalId::new(902),
        owner_id: Some(owner.id),
        name: Some("Juniper".to_owned()),
        species: None,
        birthday: None,
        unknown: BTreeMap::new(),
    };
    let item: dto::retail::Item = serde_json::from_value(serde_json::json!({
        "id": 41,
        "name": "Calming Chew",
        "sku": "CALM-CHEW",
        "category": "supplement",
        "active": true
    }))
    .unwrap();

    let owner_provenance = provenance("501", "opaque://fixture/owner", "unverified-owner-fixture");
    let animal_provenance = provenance(
        "902",
        "opaque://fixture/animal",
        "unverified-animal-fixture",
    );
    let item_provenance = provenance(
        "41",
        "opaque://fixture/retail-item",
        "unverified-retail-fixture",
    );

    let customer = mapping::customer::contact_candidate(&owner, owner_provenance).unwrap();
    let pet = mapping::pet::name_candidate(&animal, animal_provenance).unwrap();
    let retail = mapping::retail::product_candidate(&item, item_provenance).unwrap();

    assert_eq!(customer.record_ref().record_id().as_str(), "501");
    assert_eq!(
        customer.provenance().schema_version().as_str(),
        "unverified-owner-fixture"
    );
    assert_eq!(
        customer.mapping_version(),
        mapping::Version::CustomerContactV1
    );
    assert_eq!(
        customer.candidate().full_name.clone().into_inner(),
        "Sam Rivera"
    );

    assert_eq!(pet.record_ref().record_id().as_str(), "902");
    assert_eq!(pet.mapping_version(), mapping::Version::PetNameV1);
    assert_eq!(pet.candidate().name.clone().into_inner(), "Juniper");

    assert_eq!(retail.record_ref().record_id().as_str(), "41");
    assert_eq!(retail.mapping_version(), mapping::Version::RetailProductV1);
    assert_eq!(retail.candidate().product.sku().as_str(), "CALM-CHEW");
}

#[test]
fn promotions_reject_provenance_for_another_source_record() {
    let animal = response::AnimalRecord {
        id: endpoint::AnimalId::new(902),
        owner_id: None,
        name: Some("Juniper".to_owned()),
        species: None,
        birthday: None,
        unknown: BTreeMap::new(),
    };

    let error = mapping::pet::name_candidate(
        &animal,
        provenance(
            "different-animal",
            "opaque://fixture/animal",
            "unverified-animal-fixture",
        ),
    )
    .unwrap_err();

    assert_eq!(
        error,
        mapping::Error::SourceRecordMismatch {
            provider_record_id: source::record::Id::try_new("902").unwrap(),
            provenance_record_id: source::record::Id::try_new("different-animal").unwrap(),
        }
    );
}

#[test]
fn promotions_preserve_opaque_unverified_provider_contract_labels() {
    let animal = response::AnimalRecord {
        id: endpoint::AnimalId::new(902),
        owner_id: None,
        name: Some("Juniper".to_owned()),
        species: None,
        birthday: None,
        unknown: BTreeMap::new(),
    };

    let first_observation = mapping::pet::name_candidate(
        &animal,
        provenance("902", "opaque://capture/source-a", "unverified-shape-a"),
    )
    .unwrap();
    assert_eq!(
        first_observation.provenance().endpoint().as_str(),
        "opaque://capture/source-a"
    );
    assert_eq!(
        first_observation.provenance().schema_version().as_str(),
        "unverified-shape-a"
    );

    let later_observation = mapping::pet::name_candidate(
        &animal,
        provenance("902", "opaque://capture/source-b", "unverified-shape-b"),
    )
    .unwrap();
    assert_eq!(
        later_observation.provenance().endpoint().as_str(),
        "opaque://capture/source-b"
    );
    assert_eq!(
        later_observation.provenance().schema_version().as_str(),
        "unverified-shape-b"
    );
}

fn provenance(record_id: &str, endpoint: &str, schema_version: &str) -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new(endpoint).unwrap())
        .record_id(source::record::Id::try_new(record_id).unwrap())
        .extraction_batch(source::ExtractionBatchId::try_new("fixture-batch").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-08-13T21:30:00Z").unwrap())
        .request_scope(source::RequestScope::try_new("provider-promotion-contract").unwrap())
        .schema_version(source::SchemaVersion::try_new(schema_version).unwrap())
        .payload_hash(source::PayloadHash::try_new("sha256:fixture").unwrap())
        .raw_payload_ref(source::RawPayloadRef::try_new("fixture://gingr/provider-record").unwrap())
        .build()
}
