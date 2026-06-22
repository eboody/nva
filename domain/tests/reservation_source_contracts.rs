use domain::{analytics, data_quality, source};

fn gingr_provenance() -> source::gingr::Provenance {
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

fn source_provenance() -> source::Provenance {
    gingr_provenance().promote()
}

fn complete_source_snapshot() -> source::reservation::Snapshot {
    source::reservation::Snapshot::builder()
        .provenance(source_provenance())
        .customer_record_id(source::record::Id::try_new("owner-7").unwrap())
        .pet_record_id(source::record::Id::try_new("animal-9").unwrap())
        .location_record_id(source::record::Id::try_new("location-3").unwrap())
        .service_type_record_id(source::record::Id::try_new("boarding-suite").unwrap())
        .status(source::reservation::Status::CheckedIn)
        .relationship(source::reservation::OwnerPetRelationship::Resolved)
        .build()
        .unwrap()
}

#[test]
fn source_value_objects_reject_blank_values_when_rehydrated_from_storage() {
    assert!(serde_json::from_str::<source::Endpoint>("\"   \"").is_err());
    assert!(serde_json::from_str::<source::PayloadHash>("\"   \"").is_err());
    assert!(serde_json::from_str::<source::RawPayloadRef>("\"   \"").is_err());
    assert!(serde_json::from_str::<source::ObservedStatus>("\"   \"").is_err());
    assert!(serde_json::from_str::<source::gingr::ProviderStatus>("\"   \"").is_err());
}

#[test]
fn source_snapshot_builder_returns_typed_errors_for_missing_provenance_or_relationship() {
    let missing_provenance = source::reservation::Snapshot::builder()
        .relationship(source::reservation::OwnerPetRelationship::Resolved)
        .build();

    assert_eq!(
        missing_provenance,
        Err(source::Error::ReservationSnapshotProvenanceRequired)
    );

    let missing_relationship = source::reservation::Snapshot::builder()
        .provenance(source_provenance())
        .build();

    assert_eq!(
        missing_relationship,
        Err(source::Error::ReservationSnapshotRelationshipRequired)
    );
}

#[test]
fn gingr_snapshot_builder_returns_typed_errors_for_missing_source_evidence() {
    let missing_provenance = source::gingr::reservation::Snapshot::builder()
        .relationship(source::gingr::reservation::OwnerPetRelationship::Resolved)
        .build();

    assert_eq!(
        missing_provenance,
        Err(source::Error::GingrReservationSnapshotProvenanceRequired)
    );

    let missing_relationship = source::gingr::reservation::Snapshot::builder()
        .provenance(gingr_provenance())
        .build();

    assert_eq!(
        missing_relationship,
        Err(source::Error::GingrReservationSnapshotRelationshipRequired)
    );
}

#[test]
fn source_agnostic_reservation_snapshot_preserves_provenance_without_gingr_paths() {
    let snapshot = complete_source_snapshot();

    assert_eq!(snapshot.provenance().system(), source::System::Gingr);
    assert_eq!(snapshot.provenance().record_id().as_str(), "reservation-42");
    assert_eq!(
        snapshot.provenance().endpoint().as_str(),
        "GET /reservations"
    );
    assert_eq!(
        snapshot.provenance().raw_payload_ref().as_str(),
        "restricted://gingr/reservations/42.json"
    );
    assert_eq!(snapshot.customer_record_id().unwrap().as_str(), "owner-7");
    assert_eq!(
        snapshot.status(),
        Some(source::reservation::Status::CheckedIn)
    );
}

#[test]
fn gingr_reservation_snapshot_promotes_to_source_agnostic_snapshot_with_assumptions() {
    let snapshot = source::gingr::reservation::Snapshot::builder()
        .provenance(gingr_provenance())
        .owner_provider_id(source::gingr::ProviderRecordId::try_new("owner-7").unwrap())
        .animal_provider_id(source::gingr::ProviderRecordId::try_new("animal-9").unwrap())
        .location_provider_id(source::gingr::ProviderRecordId::try_new("location-3").unwrap())
        .service_type_provider_id(
            source::gingr::ProviderRecordId::try_new("boarding-suite").unwrap(),
        )
        .provider_status(source::gingr::ProviderStatus::try_new("checked_in").unwrap())
        .relationship(source::gingr::reservation::OwnerPetRelationship::Resolved)
        .build()
        .unwrap()
        .promote()
        .unwrap();

    assert_eq!(snapshot.provenance().system(), source::System::Gingr);
    assert_eq!(snapshot.provenance().record_id().as_str(), "reservation-42");
    assert_eq!(snapshot.customer_record_id().unwrap().as_str(), "owner-7");
    assert_eq!(snapshot.pet_record_id().unwrap().as_str(), "animal-9");
    assert_eq!(
        snapshot.status(),
        Some(source::reservation::Status::CheckedIn)
    );
    assert!(
        snapshot
            .assumptions()
            .contains(&source::reservation::Assumption::GrainTreatedAsReservation)
    );
    assert!(
        snapshot
            .assumptions()
            .contains(&source::reservation::Assumption::ProviderStatusMappingIsProvisional)
    );
}

#[test]
fn data_quality_issues_do_not_depend_on_gingr_provenance() {
    let issue = data_quality::Issue::new(
        data_quality::Kind::MissingRequiredField {
            field: data_quality::FieldPath::reservation(
                data_quality::ReservationField::CustomerRecordId,
            ),
        },
        data_quality::Severity::Blocking,
        source_provenance(),
        source::Timestamp::try_new("2026-06-16T20:05:00Z").unwrap(),
        true,
    );

    assert_eq!(issue.source_system(), source::System::Gingr);
    assert_eq!(issue.provenance().record_id().as_str(), "reservation-42");
    assert_eq!(
        issue.provenance().source_system(),
        issue.source_record_ref().system()
    );
    assert_eq!(
        issue.provenance().record_id(),
        issue.source_record_ref().record_id()
    );
}

#[test]
fn data_quality_field_paths_express_domain_fields_without_provider_table_names() {
    assert_eq!(
        data_quality::FieldPath::reservation(data_quality::ReservationField::Status).segments(),
        &[
            data_quality::FieldSegment::Reservation,
            data_quality::FieldSegment::Status,
        ]
    );
    assert_eq!(
        data_quality::FieldPath::stay(data_quality::StayField::LocationRecordId).segments(),
        &[
            data_quality::FieldSegment::Stay,
            data_quality::FieldSegment::LocationRecordId,
        ]
    );
    assert_eq!(
        data_quality::FieldPath::source(data_quality::SourceField::RawPayloadRef).segments(),
        &[
            data_quality::FieldSegment::Source,
            data_quality::FieldSegment::RawPayloadRef,
        ]
    );
}

#[test]
fn missing_and_ambiguous_source_facts_emit_typed_data_quality_issues() {
    let snapshot = source::reservation::Snapshot::builder()
        .provenance(source_provenance())
        .customer_record_id(None)
        .pet_record_id(source::record::Id::try_new("animal-9").unwrap())
        .location_record_id(None)
        .service_type_record_id(source::record::Id::try_new("boarding-suite").unwrap())
        .status(None)
        .relationship(source::reservation::OwnerPetRelationship::Ambiguous { candidate_count: 3 })
        .build()
        .unwrap();

    let issues =
        snapshot.data_quality_issues(source::Timestamp::try_new("2026-06-16T20:05:00Z").unwrap());

    assert!(issues.iter().any(|issue| issue.kind()
        == data_quality::Kind::MissingRequiredField {
            field: data_quality::FieldPath::reservation(
                data_quality::ReservationField::CustomerRecordId,
            ),
        }));
    assert!(issues.iter().any(|issue| issue.kind()
        == data_quality::Kind::MissingRequiredField {
            field: data_quality::FieldPath::reservation(
                data_quality::ReservationField::LocationRecordId,
            ),
        }));
    assert!(issues.iter().any(|issue| issue.kind()
        == data_quality::Kind::MissingRequiredField {
            field: data_quality::FieldPath::reservation(data_quality::ReservationField::Status),
        }));
    assert!(issues.iter().any(|issue| issue.kind()
        == data_quality::Kind::AssumptionInForce {
            assumption: source::reservation::Assumption::RefreshMutationPolicyUnknown,
        }));
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
    assert!(
        issues
            .iter()
            .filter(|issue| issue.workflow_blocking())
            .count()
            >= 4
    );
    assert!(issues.iter().filter(|issue| issue.visible_to_bi()).count() >= 4);
}

#[test]
fn complete_source_reservation_facts_project_to_stay_fact() {
    let source_reservation = complete_source_snapshot();

    let fact = analytics::stay::Fact::project_from_source_reservation(
        analytics::stay::Id::try_new("stay-fact-42").unwrap(),
        &source_reservation,
        analytics::ProjectionVersion::try_new("stay-v1").unwrap(),
    )
    .expect("complete source reservation facts can project to stay fact");

    assert_eq!(fact.id().as_str(), "stay-fact-42");
    assert_eq!(fact.source_system(), source::System::Gingr);
    assert_eq!(fact.reservation_record_id().as_str(), "reservation-42");
    assert_eq!(fact.customer_record_id().as_str(), "owner-7");
    assert_eq!(fact.pet_record_id().as_str(), "animal-9");
    assert_eq!(fact.location_record_id().as_str(), "location-3");
    assert_eq!(fact.service_type_record_id().as_str(), "boarding-suite");
    assert_eq!(fact.projection_version().as_str(), "stay-v1");
    assert_eq!(
        fact.data_quality_status(),
        analytics::stay::DataQualityStatus::Complete
    );
    assert!(fact.data_quality_issues().is_empty());
}

#[test]
fn source_reservation_projection_preserves_nonblocking_data_quality_warnings_on_stay_fact() {
    let source_reservation = source::reservation::Snapshot::builder()
        .provenance(source_provenance())
        .customer_record_id(source::record::Id::try_new("owner-7").unwrap())
        .pet_record_id(source::record::Id::try_new("animal-9").unwrap())
        .location_record_id(source::record::Id::try_new("location-3").unwrap())
        .service_type_record_id(source::record::Id::try_new("boarding-suite").unwrap())
        .status(source::reservation::Status::CheckedIn)
        .relationship(source::reservation::OwnerPetRelationship::Resolved)
        .assumptions(vec![
            source::reservation::Assumption::RawPayloadRetentionUnknown,
        ])
        .build()
        .unwrap();

    let fact = analytics::stay::Fact::project_from_source_reservation(
        analytics::stay::Id::try_new("stay-fact-warning").unwrap(),
        &source_reservation,
        analytics::ProjectionVersion::try_new("stay-v1").unwrap(),
    )
    .expect("nonblocking data-quality warnings should not block stay fact projection");

    assert_eq!(
        fact.data_quality_status(),
        analytics::stay::DataQualityStatus::ManagerReviewRequired
    );
    assert!(fact.data_quality_issues().iter().any(|issue| issue.kind()
        == data_quality::Kind::AssumptionInForce {
            assumption: source::reservation::Assumption::RawPayloadRetentionUnknown,
        }));
    assert!(fact.data_quality_issues().iter().all(|issue| {
        !issue.workflow_blocking()
            && issue.provenance() == source_reservation.provenance()
            && issue.source_record_ref().record_id() == source_reservation.provenance().record_id()
    }));
}

#[test]
fn source_specific_gingr_fixture_must_promote_before_projection_consumers_see_source_records() {
    fn assert_source_record_id(id: &source::record::Id, expected: &str) {
        assert_eq!(id.as_str(), expected);
    }

    let source_reservation = source::gingr::reservation::Snapshot::builder()
        .provenance(gingr_provenance())
        .owner_provider_id(source::gingr::ProviderRecordId::try_new("owner-7").unwrap())
        .animal_provider_id(source::gingr::ProviderRecordId::try_new("animal-9").unwrap())
        .location_provider_id(source::gingr::ProviderRecordId::try_new("location-3").unwrap())
        .service_type_provider_id(
            source::gingr::ProviderRecordId::try_new("boarding-suite").unwrap(),
        )
        .provider_status(source::gingr::ProviderStatus::try_new("checked_in").unwrap())
        .relationship(source::gingr::reservation::OwnerPetRelationship::Resolved)
        .build()
        .unwrap()
        .promote()
        .unwrap();

    let fact = analytics::stay::Fact::project_from_source_reservation(
        analytics::stay::Id::try_new("stay-fact-promoted").unwrap(),
        &source_reservation,
        analytics::ProjectionVersion::try_new("stay-v1").unwrap(),
    )
    .expect("promoted source-agnostic reservation fixture should project to stay fact");

    assert_source_record_id(fact.reservation_record_id(), "reservation-42");
    assert_source_record_id(fact.customer_record_id(), "owner-7");
    assert_source_record_id(fact.pet_record_id(), "animal-9");
    assert_source_record_id(fact.location_record_id(), "location-3");
    assert_source_record_id(fact.service_type_record_id(), "boarding-suite");
    assert_eq!(fact.source_system(), source::System::Gingr);
    assert_eq!(fact.provenance(), source_reservation.provenance());
}

#[test]
fn incomplete_source_reservation_facts_return_typed_data_quality_issues_instead_of_stay_fact() {
    let source_reservation = source::reservation::Snapshot::builder()
        .provenance(source_provenance())
        .customer_record_id(None)
        .pet_record_id(source::record::Id::try_new("animal-9").unwrap())
        .location_record_id(None)
        .service_type_record_id(source::record::Id::try_new("boarding-suite").unwrap())
        .status(source::reservation::Status::Unknown {
            observed: source::ObservedStatus::try_new("provider-specific-hold").unwrap(),
        })
        .relationship(source::reservation::OwnerPetRelationship::Resolved)
        .build()
        .unwrap();

    let issues = analytics::stay::Fact::project_from_source_reservation(
        analytics::stay::Id::try_new("stay-fact-43").unwrap(),
        &source_reservation,
        analytics::ProjectionVersion::try_new("stay-v1").unwrap(),
    )
    .expect_err("incomplete source reservation facts stay typed data-quality issues");

    assert!(issues.iter().any(|issue| issue.kind()
        == data_quality::Kind::MissingRequiredField {
            field: data_quality::FieldPath::reservation(
                data_quality::ReservationField::CustomerRecordId,
            ),
        }));
    assert!(issues.iter().any(|issue| issue.kind()
        == data_quality::Kind::MissingRequiredField {
            field: data_quality::FieldPath::reservation(
                data_quality::ReservationField::LocationRecordId,
            ),
        }));
    assert!(issues.iter().any(|issue| issue.kind()
        == data_quality::Kind::UnknownSourceStatus {
            observed: source::ObservedStatus::try_new("provider-specific-hold").unwrap(),
        }));
    assert!(issues.iter().all(data_quality::Issue::workflow_blocking));
    assert!(
        issues
            .iter()
            .all(|issue| issue.provenance() == source_reservation.provenance())
    );
}
