use chrono::{TimeZone, Utc};
use uuid::Uuid;

use app::site_finance;
use domain::{analytics, entities, money, policy, source};

#[test]
fn source_backed_site_finance_recommendation_reaches_review_action_and_claimable_outcome() {
    let slice = site_finance::fixture_site_period_projection().expect("fixture builds");

    assert_eq!(
        slice.projection().data_quality_status(),
        site_finance::DataQualityStatus::Complete
    );
    assert_eq!(
        slice.projection().net_revenue().unwrap(),
        money::Money::usd(159_000).unwrap()
    );
    assert_eq!(
        slice.recommendation().required_review_gate(),
        policy::ReviewGate::ManagerApproval
    );
    assert!(slice.recommendation().blocks_financial_mutation());
    assert_eq!(
        slice.action().legal_action(),
        site_finance::LegalFinanceAction::RecordReviewedRecommendationOnly
    );
    assert!(!slice.action().allows_financial_mutation());
    assert_eq!(slice.action().source_record_refs().len(), 2);
    assert!(slice.strong_outcome().can_support_value_claim());
    assert!(!slice.weak_outcome().can_support_value_claim());
}

#[test]
fn value_claim_support_does_not_imply_workflow_approval_or_payment_authority() {
    let slice = site_finance::fixture_site_period_projection().expect("fixture builds");

    assert!(slice.strong_outcome().can_support_value_claim());
    assert_eq!(
        slice.recommendation().required_review_gate(),
        policy::ReviewGate::ManagerApproval
    );
    assert!(!slice.action().allows_financial_mutation());
}

#[test]
fn site_finance_projection_rejects_mismatched_site_period_currency_and_underflow() {
    let mut request = site_finance_request();
    request.benchmark_period = site_period(other_location_id()).unwrap();
    assert_eq!(
        site_finance::Workflow::evaluate(request).unwrap_err(),
        site_finance::Error::SitePeriodMismatch
    );

    let mut request = site_finance_request();
    request.benchmark = request.benchmark.with_gross_revenue(money::Money::new(
        money::MinorUnits::try_new(190_000).unwrap(),
        money::Currency::AccountingSystem("CAD".to_owned()),
    ));
    assert_eq!(
        site_finance::Workflow::evaluate(request).unwrap_err(),
        site_finance::Error::CurrencyMismatch
    );

    let mut request = site_finance_request();
    request.current = request
        .current
        .with_discount(money::Money::usd(300_000).unwrap());
    assert_eq!(
        site_finance::Workflow::evaluate(request).unwrap_err(),
        site_finance::Error::ArithmeticWouldUnderflow
    );
}

fn site_finance_request() -> site_finance::Request {
    site_finance::Request {
        current: site_finance::SourceBackedFinancialFact::builder()
            .period(site_period(location_id()).unwrap())
            .service(entities::ServiceKind::Boarding)
            .gross_revenue(money::Money::usd(180_000).unwrap())
            .discount(money::Money::usd(15_000).unwrap())
            .refund(money::Money::usd(6_000).unwrap())
            .labor_cost(money::Money::usd(74_000).unwrap())
            .source_refs(source_refs("current"))
            .projection_version(analytics::ProjectionVersion::try_new("site-finance-v1").unwrap())
            .data_quality_issues(vec![])
            .build(),
        benchmark: site_finance::SourceBackedFinancialFact::builder()
            .period(site_period(location_id()).unwrap())
            .service(entities::ServiceKind::Boarding)
            .gross_revenue(money::Money::usd(173_000).unwrap())
            .discount(money::Money::usd(8_000).unwrap())
            .refund(money::Money::usd(2_000).unwrap())
            .labor_cost(money::Money::usd(71_000).unwrap())
            .source_refs(source_refs("benchmark"))
            .projection_version(analytics::ProjectionVersion::try_new("site-finance-v1").unwrap())
            .data_quality_issues(vec![])
            .build(),
        benchmark_period: site_period(location_id()).unwrap(),
    }
}

fn site_period(
    location_id: entities::LocationId,
) -> std::result::Result<domain::analytics::finance::SitePeriod, domain::analytics::finance::Error>
{
    domain::analytics::finance::SitePeriod::builder()
        .location_id(location_id)
        .start(Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap())
        .end(Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap())
        .build()
}

fn location_id() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(0x00c0ffee000000000000000000000001))
}

fn other_location_id() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(0x00c0ffee000000000000000000000002))
}

fn source_refs(suffix: &str) -> Vec<source::RecordRef> {
    vec![source::RecordRef::new(
        source::System::FinanceAccounting,
        source::record::Id::try_new(format!("site-finance-boarding-2026-06-{suffix}")).unwrap(),
    )]
}
