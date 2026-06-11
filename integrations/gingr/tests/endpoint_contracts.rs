use gingr::{config, endpoint, transport};

const SENTINEL_KEY: &str = "gingr_test_api_key_do_not_send";

fn fake_client() -> transport::Client<transport::MockTransport> {
    let config = config::ClientConfig::new(
        config::BaseUrl::parse("https://example-pet-resort.gingrapp.com").unwrap(),
        config::ApiKey::from_secret(SENTINEL_KEY),
    );
    transport::Client::with_transport(config, transport::MockTransport)
}

#[test]
fn get_locations_is_a_get_request_with_api_key_added_by_transport() {
    let client = fake_client();
    let request = endpoint::reference_data::GetLocations;

    let redacted = client.redacted_request(&request).unwrap();
    let sent = client.capture_request(&request).unwrap();

    assert_eq!(sent.method(), endpoint::Method::Get);
    assert_eq!(sent.path(), "/api/v1/get_locations");
    assert!(
        sent.query_pairs()
            .iter()
            .any(|(key, value)| key == "key" && value == SENTINEL_KEY)
    );
    assert!(!redacted.to_string().contains(SENTINEL_KEY));
    assert!(redacted.to_string().contains("key=<redacted>"));
}

#[test]
fn reservation_types_keeps_optional_filters_typed_and_redacted_diagnostics_safe() {
    let client = fake_client();
    let request = endpoint::reservations::ReservationTypes::builder()
        .id(endpoint::reservations::ReservationTypeId::new(42))
        .active_only(true)
        .build();

    let sent = client.capture_request(&request).unwrap();
    let redacted = client.redacted_request(&request).unwrap().to_string();

    assert_eq!(sent.method(), endpoint::Method::Get);
    assert_eq!(sent.path(), "/api/v1/reservation_types");
    assert!(sent.query_pairs().contains(&("id".into(), "42".into())));
    assert!(
        sent.query_pairs()
            .contains(&("active_only".into(), "true".into()))
    );
    assert!(!redacted.contains(SENTINEL_KEY));
}

#[test]
fn reservation_widget_data_requires_a_yyyy_mm_dd_date_parameter() {
    let client = fake_client();
    let date = endpoint::Date::parse("2026-06-10").unwrap();
    let request = endpoint::reservations::ReservationWidgetData::builder()
        .timestamp(date)
        .build();

    let sent = client.capture_request(&request).unwrap();

    assert_eq!(sent.method(), endpoint::Method::Get);
    assert_eq!(sent.path(), "/api/v1/reservation_widget_data");
    assert!(
        sent.query_pairs()
            .contains(&("timestamp".into(), "2026-06-10".into()))
    );
    assert!(endpoint::Date::parse("06/10/2026").is_err());
}

#[test]
fn owners_and_animals_filters_are_quarantined_as_provider_where_clauses() {
    let client = fake_client();
    let owner_request = endpoint::owners_animals::Owners::builder()
        .provider_where_clause(endpoint::owners_animals::ProviderWhereClause::new(
            "zip", "80302",
        ))
        .build();
    let animal_request = endpoint::owners_animals::Animals::builder()
        .provider_where_clause(endpoint::owners_animals::ProviderWhereClause::new(
            "month(from_unixtime(birthday))",
            "11",
        ))
        .build();

    let owner_sent = client.capture_request(&owner_request).unwrap();
    let animal_sent = client.capture_request(&animal_request).unwrap();

    assert_eq!(owner_sent.path(), "/api/v1/owners");
    assert!(
        owner_sent
            .query_pairs()
            .contains(&("params[zip]".into(), "80302".into()))
    );
    assert_eq!(animal_sent.path(), "/api/v1/animals");
    assert!(
        animal_sent
            .query_pairs()
            .contains(&("params[month(from_unixtime(birthday))]".into(), "11".into()))
    );
}

#[test]
fn excluded_side_effect_endpoints_are_not_exported_as_request_builders() {
    let exported_endpoint_names = endpoint::catalog::exported_read_endpoint_names();

    assert!(exported_endpoint_names.contains(&"get_locations"));
    assert!(exported_endpoint_names.contains(&"reservation_types"));
    assert!(exported_endpoint_names.contains(&"reservation_widget_data"));
    assert!(exported_endpoint_names.contains(&"owners"));
    assert!(exported_endpoint_names.contains(&"animals"));
    assert!(!exported_endpoint_names.contains(&"quick_checkin"));
    assert!(!exported_endpoint_names.contains(&"receive_call"));
}

#[test]
fn commerce_retail_endpoints_preserve_legacy_date_boundaries_and_payment_sensitivity() {
    let client = fake_client();
    let before_legacy_cutover = endpoint::Date::parse("2019-07-31").unwrap();
    let on_invoice_cutover = endpoint::Date::parse("2019-08-01").unwrap();

    let retail_items = client
        .capture_request(&endpoint::commerce_retail::GetAllRetailItems)
        .unwrap();
    assert_eq!(retail_items.method(), endpoint::Method::Get);
    assert_eq!(retail_items.path(), "/api/v1/get_all_retail_items");

    let transactions = endpoint::commerce_retail::ListTransactions::builder()
        .from_date(before_legacy_cutover)
        .to_date(before_legacy_cutover)
        .build()
        .unwrap();
    let sent_transactions = client.capture_request(&transactions).unwrap();
    assert_eq!(sent_transactions.path(), "/api/v1/list_transactions");
    assert!(
        sent_transactions
            .query_pairs()
            .contains(&("from_date".into(), "2019-07-31".into()))
    );
    assert!(
        endpoint::commerce_retail::ListTransactions::builder()
            .from_date(on_invoice_cutover)
            .to_date(on_invoice_cutover)
            .build()
            .is_err(),
        "list_transactions is documented for pre-2019-08-01 POS transactions only"
    );

    let transaction = endpoint::commerce_retail::Transaction::new(
        endpoint::commerce_retail::TransactionId::new(12345),
    );
    let sent_transaction = client.capture_request(&transaction).unwrap();
    assert_eq!(sent_transaction.method(), endpoint::Method::Post);
    assert_eq!(sent_transaction.path(), "/api/v1/transaction");
    assert!(
        sent_transaction
            .form_pairs()
            .contains(&("id".into(), "12345".into()))
    );
    assert_eq!(
        transaction.sensitivity(),
        endpoint::commerce_retail::ResponseSensitivity::PaymentSensitive
    );
}

#[test]
fn list_invoices_requires_paired_pagination_and_on_or_after_legacy_cutover_dates() {
    let client = fake_client();
    let cutover = endpoint::Date::parse("2019-08-01").unwrap();

    let invoices = endpoint::commerce_retail::ListInvoices::builder()
        .pagination(endpoint::commerce_retail::InvoicePagination::new(10, 21).unwrap())
        .complete(true)
        .closed_only(true)
        .from_date(cutover)
        .to_date(cutover)
        .build()
        .unwrap();

    let sent = client.capture_request(&invoices).unwrap();
    assert_eq!(sent.path(), "/api/v1/list_invoices");
    assert!(
        sent.query_pairs()
            .contains(&("per_page".into(), "10".into()))
    );
    assert!(sent.query_pairs().contains(&("page".into(), "21".into())));
    assert!(
        sent.query_pairs()
            .contains(&("complete".into(), "true".into()))
    );
    assert!(
        sent.query_pairs()
            .contains(&("closed_only".into(), "true".into()))
    );
    assert!(
        sent.query_pairs()
            .contains(&("from_date".into(), "2019-08-01".into()))
    );

    assert!(endpoint::commerce_retail::InvoicePagination::new(10, 20).is_err());
    assert!(
        endpoint::commerce_retail::ListInvoices::builder()
            .from_date(endpoint::Date::parse("2019-07-31").unwrap())
            .build()
            .is_err(),
        "list_invoices is documented for invoices created on/after 2019-08-01"
    );
}

#[test]
fn subscriptions_timeclock_and_report_card_files_expose_documented_filters() {
    let client = fake_client();

    let subscription = endpoint::commerce_retail::GetSubscription::new(
        endpoint::commerce_retail::SubscriptionId::new(77),
    );
    let sent_subscription = client.capture_request(&subscription).unwrap();
    assert_eq!(sent_subscription.path(), "/api/v1/get_subscription");
    assert!(
        sent_subscription
            .query_pairs()
            .contains(&("id".into(), "77".into()))
    );

    let subscriptions = endpoint::commerce_retail::GetSubscriptions::builder()
        .include_deleted(true)
        .bill_day_of_month(endpoint::commerce_retail::BillDayOfMonth::new(15).unwrap())
        .owner_id(endpoint::OwnerId::new(42))
        .pagination(endpoint::commerce_retail::SubscriptionPagination::new(
            50, 100,
        ))
        .location_id(endpoint::LocationId::new(3))
        .package_id(endpoint::commerce_retail::PackageId::new(9))
        .build();
    let sent_subscriptions = client.capture_request(&subscriptions).unwrap();
    assert_eq!(sent_subscriptions.path(), "/api/v1/get_subscriptions");
    assert!(
        sent_subscriptions
            .query_pairs()
            .contains(&("limit".into(), "50".into()))
    );
    assert!(
        sent_subscriptions
            .query_pairs()
            .contains(&("offset".into(), "100".into()))
    );
    assert!(
        sent_subscriptions
            .query_pairs()
            .contains(&("owner_id".into(), "42".into()))
    );
    assert!(
        sent_subscriptions
            .query_pairs()
            .contains(&("location_id".into(), "3".into()))
    );

    let timeclock = endpoint::labor_ops::TimeclockReport::builder()
        .date_range(
            endpoint::Date::parse("2026-06-01").unwrap(),
            endpoint::Date::parse("2026-06-10").unwrap(),
        )
        .location_id(endpoint::LocationId::new(3))
        .include_deleted(true)
        .include_clocked_in(false)
        .user_id(endpoint::labor_ops::UserId::new(12))
        .user_id(endpoint::labor_ops::UserId::new(34))
        .build()
        .unwrap();
    let sent_timeclock = client.capture_request(&timeclock).unwrap();
    assert_eq!(sent_timeclock.path(), "/api/v1/timeclock_report");
    assert!(
        sent_timeclock
            .query_pairs()
            .contains(&("start_date".into(), "2026-06-01".into()))
    );
    assert!(
        sent_timeclock
            .query_pairs()
            .contains(&("end_date".into(), "2026-06-10".into()))
    );
    assert!(
        sent_timeclock
            .query_pairs()
            .contains(&("location_id".into(), "3".into()))
    );
    assert!(
        sent_timeclock
            .query_pairs()
            .contains(&("user_ids[]".into(), "12".into()))
    );
    assert!(
        sent_timeclock
            .query_pairs()
            .contains(&("user_ids[]".into(), "34".into()))
    );

    let report_cards = endpoint::report_cards_files::ReportCardFiles::builder()
        .number_days(14)
        .limit(25)
        .location_id(endpoint::LocationId::new(3))
        .build();
    let sent_report_cards = client.capture_request(&report_cards).unwrap();
    assert_eq!(sent_report_cards.path(), "/api/v1/report_card_files");
    assert!(
        sent_report_cards
            .query_pairs()
            .contains(&("number_days".into(), "14".into()))
    );
    assert!(
        sent_report_cards
            .query_pairs()
            .contains(&("limit".into(), "25".into()))
    );
    assert!(
        sent_report_cards
            .query_pairs()
            .contains(&("location_id".into(), "3".into()))
    );
}

#[test]
fn expanded_read_catalog_documents_mapping_gaps_for_unmodeled_provider_domains() {
    let exported_endpoint_names = endpoint::catalog::exported_read_endpoint_names();

    for endpoint_name in [
        "get_all_retail_items",
        "list_transactions",
        "transaction",
        "list_invoices",
        "get_subscription",
        "get_subscriptions",
        "timeclock_report",
        "report_card_files",
    ] {
        assert!(exported_endpoint_names.contains(&endpoint_name));
    }

    let gaps = endpoint::catalog::semantic_mapping_gaps();
    assert!(gaps.contains(&"retail"));
    assert!(gaps.contains(&"training"));
    assert!(gaps.contains(&"grooming"));
}
