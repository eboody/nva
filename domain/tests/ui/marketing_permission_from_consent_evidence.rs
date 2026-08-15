use chrono::{TimeZone, Utc};
use domain::{access, consent, customer, entities, source};

fn main() {
    let now = Utc.with_ymd_and_hms(2026, 8, 15, 0, 0, 0).unwrap();
    let customer_id = entities::CustomerId::new(uuid::Uuid::from_u128(1));
    let evidence = consent::ConsentEvidence::builder()
        .channel(consent::Channel::Email)
        .purpose(consent::Purpose::MarketingRetention)
        .status(consent::ConsentStatus::Granted)
        .source(source::System::Crm)
        .subject(consent::Subject::Customer(customer_id))
        .source_record(source::RecordRef::new(
            source::System::Crm,
            source::record::Id::try_new("consent-1").unwrap(),
        ))
        .source_schema_version(source::SchemaVersion::try_new("v1").unwrap())
        .effective_from(now)
        .build();
    let membership: customer::intelligence::SegmentMembership = todo!();

    let _ = customer::intelligence::MarketingUsePermission::try_from_membership(
        &membership,
        Some(evidence),
        access::AllowedUse::MarketingCampaign,
        now,
    );
}
