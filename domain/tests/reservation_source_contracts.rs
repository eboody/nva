use domain::{analytics, data_quality, source};

fn provenance() -> source::gingr::Provenance {
    source::gingr::Provenance::builder()
        .endpoint(source::gingr::Endpoint::try_new("GET /reservations").unwrap())
        .provider_record_id(source::gingr::ProviderRecordId::try_new("reservation-42").unwrap())
        .related_provider_ids(vec![
            source::gingr::RelatedProviderId::owner(
                source::gingr::ProviderRecordId::try_new("owner-7").unwrap(),
            ),
            source::gingr::RelatedProviderId::animal(
                source::gingr::ProviderRecordId::try_new("animal-9").unwrap(),
            ),
        ])
        .extraction_batch(source::gingr::ExtractionBatchId::try_new("batch-2026-06-16").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-06-16T20:00:00Z").unwrap())
        .request_scope(
            source::gingr::RequestScope::try_new(
                "account=nva-demo; location=west-loop; window=2026-06-01..2026-06-30",
            )
            .unwrap(),
        )
        .provider_schema_version(
            source::gingr::ProviderSchemaVersion::try_new("local-parser-v1").unwrap(),
        )
        .source_payload_hash(source::PayloadHash::try_new("sha256:reservation42").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("restricted://gingr/reservations/42.json").unwrap(),
        )
        .build()
}

#[test]
fn gingr_reservation_snapshot_preserves_provider_provenance() {
    let snapshot = source::gingr::reservation::Snapshot::builder()
        .provenance(provenance())
        .owner_provider_id(source::gingr::ProviderRecordId::try_new("owner-7").unwrap())
        .animal_provider_id(source::gingr::ProviderRecordId::try_new("animal-9").unwrap())
        .location_provider_id(source::gingr::ProviderRecordId::try_new("location-3").unwrap())
        .service_type_provider_id(
            source::gingr::ProviderRecordId::try_new("boarding-suite").unwrap(),
        )
        .provider_status(source::gingr::ProviderStatus::try_new("checked_in").unwrap())
        .relationship(source::gingr::reservation::OwnerPetRelationship::Resolved)
        .build();

    assert_eq!(snapshot.provenance().source_system(), source::System::Gingr);
    assert_eq!(
        snapshot.provenance().provider_record_id().as_str(),
        "reservation-42"
    );
    assert_eq!(
        snapshot.provenance().endpoint().as_str(),
        "GET /reservations"
    );
    assert_eq!(
        snapshot.provenance().raw_payload_ref().as_str(),
        "restricted://gingr/reservations/42.json"
    );
    assert_eq!(snapshot.owner_provider_id().unwrap().as_str(), "owner-7");
    assert_eq!(snapshot.provider_status().unwrap().as_str(), "checked_in");
}

#[test]
fn missing_and_ambiguous_source_facts_emit_typed_data_quality_issues() {
    let snapshot = source::gingr::reservation::Snapshot::builder()
        .provenance(provenance())
        .owner_provider_id(None)
        .animal_provider_id(source::gingr::ProviderRecordId::try_new("animal-9").unwrap())
        .location_provider_id(None)
        .service_type_provider_id(
            source::gingr::ProviderRecordId::try_new("boarding-suite").unwrap(),
        )
        .provider_status(None)
        .relationship(
            source::gingr::reservation::OwnerPetRelationship::Ambiguous { candidate_count: 3 },
        )
        .build();

    let issues =
        snapshot.data_quality_issues(source::Timestamp::try_new("2026-06-16T20:05:00Z").unwrap());

    assert!(issues.iter().any(|issue| issue.kind()
        == data_quality::Kind::MissingRequiredField {
            field: data_quality::SourceField::OwnerProviderId,
        }));
    assert!(issues.iter().any(|issue| issue.kind()
        == data_quality::Kind::MissingRequiredField {
            field: data_quality::SourceField::LocationProviderId,
        }));
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind() == data_quality::Kind::UnknownProviderStatus)
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind() == data_quality::Kind::AmbiguousOwnerPetRelationship)
    );
    assert!(
        issues
            .iter()
            .all(|issue| issue.source_system() == source::System::Gingr)
    );
    assert!(issues.iter().any(|issue| issue.workflow_blocking()));
}

#[test]
fn complete_gingr_reservation_snapshot_projects_to_bi_stay_fact() {
    let snapshot = source::gingr::reservation::Snapshot::builder()
        .provenance(provenance())
        .owner_provider_id(source::gingr::ProviderRecordId::try_new("owner-7").unwrap())
        .animal_provider_id(source::gingr::ProviderRecordId::try_new("animal-9").unwrap())
        .location_provider_id(source::gingr::ProviderRecordId::try_new("location-3").unwrap())
        .service_type_provider_id(
            source::gingr::ProviderRecordId::try_new("boarding-suite").unwrap(),
        )
        .provider_status(source::gingr::ProviderStatus::try_new("checked_in").unwrap())
        .relationship(source::gingr::reservation::OwnerPetRelationship::Resolved)
        .build();

    let fact = analytics::stay::Fact::project_from_gingr_reservation(
        analytics::stay::Id::try_new("stay-fact-42").unwrap(),
        &snapshot,
        analytics::ProjectionVersion::try_new("stay-v1").unwrap(),
    )
    .expect("complete snapshot can project to stay fact");

    assert_eq!(fact.id().as_str(), "stay-fact-42");
    assert_eq!(fact.source_system(), source::System::Gingr);
    assert_eq!(fact.reservation_provider_id().as_str(), "reservation-42");
    assert_eq!(fact.pet_provider_id().as_str(), "animal-9");
    assert_eq!(fact.location_provider_id().as_str(), "location-3");
    assert_eq!(fact.projection_version().as_str(), "stay-v1");
    assert_eq!(
        fact.data_quality_status(),
        analytics::stay::DataQualityStatus::Complete
    );
}
