use chrono::{TimeZone, Utc};
use domain::{analytics, data_quality, entities, money, policy, source};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Data-quality status for source-backed site finance projections.
pub enum DataQualityStatus {
    /// Source facts validated cleanly and can support reviewed recommendations.
    Complete,
    /// Nonblocking source issues remain visible to manager review.
    ManagerReviewRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Source-backed, currency-aware finance fact for one site/service/period.
pub struct SourceBackedFinancialFact {
    period: analytics::finance::SitePeriod,
    service: entities::ServiceKind,
    gross_revenue: money::Money,
    discount: money::Money,
    refund: money::Money,
    labor_cost: money::Money,
    source_refs: Vec<source::RecordRef>,
    projection_version: analytics::ProjectionVersion,
    #[builder(default)]
    data_quality_issues: Vec<data_quality::Issue>,
}

impl SourceBackedFinancialFact {
    /// Returns a copy with a changed gross revenue value for contract tests and fixture variants.
    pub fn with_gross_revenue(mut self, value: money::Money) -> Self {
        self.gross_revenue = value;
        self
    }

    /// Returns a copy with a changed discount value for contract tests and fixture variants.
    pub fn with_discount(mut self, value: money::Money) -> Self {
        self.discount = value;
        self
    }

    /// Reporting period for site/period relationship checks.
    pub const fn period(&self) -> &analytics::finance::SitePeriod {
        &self.period
    }

    /// Source refs that prove this projection is not invented.
    pub fn source_refs(&self) -> &[source::RecordRef] {
        &self.source_refs
    }

    /// Data-quality status derived from attached nonblocking issues.
    pub fn data_quality_status(&self) -> DataQualityStatus {
        if self.data_quality_issues.is_empty() {
            DataQualityStatus::Complete
        } else {
            DataQualityStatus::ManagerReviewRequired
        }
    }

    /// Net revenue after discounts/refunds, using checked currency-aware arithmetic.
    pub fn net_revenue(&self) -> money::Result<money::Money> {
        self.gross_revenue
            .checked_sub(self.discount.clone())?
            .checked_sub(self.refund.clone())
    }

    fn labor_cost(&self) -> &money::Money {
        &self.labor_cost
    }

    fn service(&self) -> entities::ServiceKind {
        self.service.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Input pair for comparing current site finance evidence to a benchmark.
pub struct Request {
    /// Current period/site/service finance evidence under review.
    pub current: SourceBackedFinancialFact,
    /// Comparable benchmark evidence used to calculate reviewed variance.
    pub benchmark: SourceBackedFinancialFact,
    /// Explicit benchmark period relationship check; prevents accidental cross-period comparisons.
    pub benchmark_period: analytics::finance::SitePeriod,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Relationship-checked current-vs-benchmark site finance projection.
pub struct Projection {
    current: SourceBackedFinancialFact,
    benchmark: SourceBackedFinancialFact,
    variance: money::Money,
}

impl Projection {
    /// Current fact after relationship and arithmetic checks.
    pub const fn current(&self) -> &SourceBackedFinancialFact {
        &self.current
    }

    /// Data-quality status visible to manager review.
    pub fn data_quality_status(&self) -> DataQualityStatus {
        self.current.data_quality_status()
    }

    /// Current net revenue after checked discount/refund math.
    pub fn net_revenue(&self) -> money::Result<money::Money> {
        self.current.net_revenue()
    }

    /// Positive current-vs-benchmark variance used only as reviewed recommendation evidence.
    pub const fn variance(&self) -> &money::Money {
        &self.variance
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Manager-review-gated finance recommendation that cannot mutate money state.
pub struct ReviewedRecommendation {
    projection: Projection,
    review_gate: policy::ReviewGate,
    recommendation: analytics::finance::Recommendation,
}

impl ReviewedRecommendation {
    /// Builds a finance recommendation only behind manager approval.
    pub fn try_new(
        projection: Projection,
        recommendation: analytics::finance::Recommendation,
        review_gate: policy::ReviewGate,
    ) -> Result<Self> {
        if review_gate != policy::ReviewGate::ManagerApproval {
            return Err(Error::ManagerReviewRequired);
        }
        Ok(Self {
            projection,
            review_gate,
            recommendation,
        })
    }

    /// Review gate required before a recommendation can be acted on by staff.
    pub fn required_review_gate(&self) -> policy::ReviewGate {
        self.review_gate.clone()
    }

    /// Finance recommendation kind retained for API/storage mapping.
    pub const fn recommendation(&self) -> analytics::finance::Recommendation {
        self.recommendation
    }

    /// Financial recommendations cannot mutate price, discount, refund, payment, or accounting state.
    pub const fn blocks_financial_mutation(&self) -> bool {
        true
    }

    /// Projection evidence behind the recommendation.
    pub const fn projection(&self) -> &Projection {
        &self.projection
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Legal finance action available in the no-side-effect pilot.
pub enum LegalFinanceAction {
    /// Record that a manager reviewed the recommendation; do not move money or provider/accounting state.
    RecordReviewedRecommendationOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Reviewed action proof binding recommendation, review packet, audit event, and source evidence.
pub struct ReviewedFinanceAction {
    recommendation: ReviewedRecommendation,
    review_packet_id: analytics::outcome::RecommendationRef,
    audit_event_id: analytics::outcome::EvidenceRef,
    source_record_refs: Vec<source::RecordRef>,
    legal_action: LegalFinanceAction,
}

impl ReviewedFinanceAction {
    /// Promotes a manager-reviewed recommendation into a record-only legal action proof.
    pub fn try_from_review(
        recommendation: ReviewedRecommendation,
        review_packet_id: analytics::outcome::RecommendationRef,
        audit_event_id: analytics::outcome::EvidenceRef,
    ) -> Result<Self> {
        if recommendation.required_review_gate() != policy::ReviewGate::ManagerApproval {
            return Err(Error::ManagerReviewRequired);
        }
        let source_record_refs = recommendation
            .projection()
            .current()
            .source_refs()
            .iter()
            .chain(recommendation.projection().benchmark.source_refs().iter())
            .cloned()
            .collect();
        Ok(Self {
            recommendation,
            review_packet_id,
            audit_event_id,
            source_record_refs,
            legal_action: LegalFinanceAction::RecordReviewedRecommendationOnly,
        })
    }

    /// Legal action exposed by this proof token.
    pub const fn legal_action(&self) -> LegalFinanceAction {
        self.legal_action
    }

    /// Money/provider/accounting mutation is intentionally unavailable in this pilot action.
    pub const fn allows_financial_mutation(&self) -> bool {
        false
    }

    /// Review packet id that must survive storage/API mapping.
    pub const fn review_packet_id(&self) -> &analytics::outcome::RecommendationRef {
        &self.review_packet_id
    }

    /// Audit event id that must survive storage/API mapping.
    pub const fn audit_event_id(&self) -> &analytics::outcome::EvidenceRef {
        &self.audit_event_id
    }

    /// Source evidence attached to the reviewed action.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// End-to-end fixture result for site finance recommendation and outcome proof.
pub struct FixtureSlice {
    projection: Projection,
    recommendation: ReviewedRecommendation,
    action: ReviewedFinanceAction,
    reviewed_action_evidence_outcome: analytics::outcome::Record,
    correlated_evidence_outcome: analytics::outcome::Record,
}

impl FixtureSlice {
    /// Relationship-checked finance projection fixture.
    pub const fn projection(&self) -> &Projection {
        &self.projection
    }

    /// Manager-review-gated recommendation fixture.
    pub const fn recommendation(&self) -> &ReviewedRecommendation {
        &self.recommendation
    }

    /// Record-only legal action proof fixture.
    pub const fn action(&self) -> &ReviewedFinanceAction {
        &self.action
    }

    /// Serializable reviewed-action evidence that remains non-claimable without accepted authority.
    pub const fn reviewed_action_evidence_outcome(&self) -> &analytics::outcome::Record {
        &self.reviewed_action_evidence_outcome
    }

    /// Correlated-only outcome that remains non-claimable evidence.
    pub const fn correlated_evidence_outcome(&self) -> &analytics::outcome::Record {
        &self.correlated_evidence_outcome
    }
}

/// Deterministic workflow evaluator for site finance projection/recommendation proof.
pub struct Workflow;

impl Workflow {
    /// Evaluates current and benchmark facts after enforcing site/period, service, source, and currency relationships.
    pub fn evaluate(request: Request) -> Result<Projection> {
        if request.current.period() != &request.benchmark_period
            || request.benchmark.period() != &request.benchmark_period
            || request.current.period() != request.benchmark.period()
        {
            return Err(Error::SitePeriodMismatch);
        }
        if request.current.service() != request.benchmark.service() {
            return Err(Error::ServiceMismatch);
        }
        if request.current.source_refs().is_empty() || request.benchmark.source_refs().is_empty() {
            return Err(Error::MissingSourceEvidence);
        }

        let current_net = request.current.net_revenue().map_err(map_money_error)?;
        let benchmark_net = request.benchmark.net_revenue().map_err(map_money_error)?;
        let variance = current_net
            .checked_sub(benchmark_net)
            .map_err(map_money_error)?;
        let _labor_delta = request
            .current
            .labor_cost()
            .clone()
            .checked_sub(request.benchmark.labor_cost().clone())
            .map_err(map_money_error)?;

        Ok(Projection {
            current: request.current,
            benchmark: request.benchmark,
            variance,
        })
    }
}

/// Builds the site-finance vertical-slice fixture through recommendation, action, and outcome proof.
pub fn fixture_site_period_projection() -> Result<FixtureSlice> {
    let projection = Workflow::evaluate(fixture_request())?;
    let recommendation = ReviewedRecommendation::try_new(
        projection.clone(),
        analytics::finance::Recommendation::ReviewLaborPlan,
        policy::ReviewGate::ManagerApproval,
    )?;
    let review_packet_id =
        analytics::outcome::RecommendationRef::try_new("site-finance-review:00c0ffee:2026-06")
            .expect("static review id is valid");
    let audit_event_id =
        analytics::outcome::EvidenceRef::try_new("audit:site-finance-review:00c0ffee:2026-06")
            .expect("static audit id is valid");
    let action = ReviewedFinanceAction::try_from_review(
        recommendation.clone(),
        review_packet_id.clone(),
        audit_event_id.clone(),
    )?;
    let reviewed_action_evidence_outcome = analytics::outcome::Record::builder()
        .id(analytics::outcome::Id::try_new("site-finance-outcome-strong").unwrap())
        .workstream(analytics::outcome::Workstream::FinancialInsights)
        .location_id(location_id())
        .change(analytics::outcome::MeasuredChange::revenue(
            money::Money::usd(159_000).unwrap(),
            money::Money::usd(166_000).unwrap(),
        ))
        .attribution(
            analytics::outcome::AttributionEvidence::reported_reviewed_action(
                review_packet_id,
                audit_event_id,
            ),
        )
        .source(source::System::FinanceAccounting)
        .recorded_at(Utc.with_ymd_and_hms(2026, 7, 2, 12, 0, 0).unwrap())
        .build();
    let correlated_evidence_outcome = analytics::outcome::Record::builder()
        .id(analytics::outcome::Id::try_new("site-finance-outcome-weak").unwrap())
        .workstream(analytics::outcome::Workstream::FinancialInsights)
        .location_id(location_id())
        .change(analytics::outcome::MeasuredChange::revenue(
            money::Money::usd(159_000).unwrap(),
            money::Money::usd(166_000).unwrap(),
        ))
        .attribution(analytics::outcome::AttributionEvidence::correlated_only(
            analytics::outcome::EvidenceRef::try_new("correlation-only:site-finance:2026-06")
                .unwrap(),
        ))
        .source(source::System::BusinessIntelligence)
        .recorded_at(Utc.with_ymd_and_hms(2026, 7, 2, 12, 0, 0).unwrap())
        .build();

    Ok(FixtureSlice {
        projection,
        recommendation,
        action,
        reviewed_action_evidence_outcome,
        correlated_evidence_outcome,
    })
}

fn fixture_request() -> Request {
    Request {
        current: SourceBackedFinancialFact::builder()
            .period(site_period(location_id()).unwrap())
            .service(entities::ServiceKind::Boarding)
            .gross_revenue(money::Money::usd(180_000).unwrap())
            .discount(money::Money::usd(15_000).unwrap())
            .refund(money::Money::usd(6_000).unwrap())
            .labor_cost(money::Money::usd(74_000).unwrap())
            .source_refs(vec![source_ref("current")])
            .projection_version(analytics::ProjectionVersion::try_new("site-finance-v1").unwrap())
            .build(),
        benchmark: SourceBackedFinancialFact::builder()
            .period(site_period(location_id()).unwrap())
            .service(entities::ServiceKind::Boarding)
            .gross_revenue(money::Money::usd(160_000).unwrap())
            .discount(money::Money::usd(8_000).unwrap())
            .refund(money::Money::usd(2_000).unwrap())
            .labor_cost(money::Money::usd(71_000).unwrap())
            .source_refs(vec![source_ref("benchmark")])
            .projection_version(analytics::ProjectionVersion::try_new("site-finance-v1").unwrap())
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
    entities::LocationId::new(Uuid::from_u128(0x00c0ffee000000000000000000000001))
}

fn source_ref(suffix: &str) -> source::RecordRef {
    source::RecordRef::new(
        source::System::FinanceAccounting,
        source::record::Id::try_new(format!("site-finance-boarding-2026-06-{suffix}")).unwrap(),
    )
}

fn map_money_error(error: money::Error) -> Error {
    match error {
        money::Error::CurrencyMismatch { .. } => Error::CurrencyMismatch,
        money::Error::SubtractionWouldBeNegative => Error::ArithmeticWouldUnderflow,
        money::Error::AdditionOverflow | money::Error::AmountOverflow => Error::ArithmeticOverflow,
        money::Error::NegativeAmount | money::Error::BasisPointsOutOfRange => Error::InvalidMoney,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Site finance workflow validation failures.
pub enum Error {
    #[error("site finance comparison requires the same site and period")]
    /// Current and benchmark evidence do not describe the same location/period.
    SitePeriodMismatch,
    #[error("site finance comparison requires the same service")]
    /// Current and benchmark evidence do not describe the same service line.
    ServiceMismatch,
    #[error("site finance projection requires source evidence")]
    /// Current or benchmark evidence lacks source record references.
    MissingSourceEvidence,
    #[error("site finance comparison requires matching currencies")]
    /// Finance facts cannot be compared because their money values use different currencies.
    CurrencyMismatch,
    #[error("site finance arithmetic would underflow")]
    /// Checked revenue/labor arithmetic would produce a negative value where the type forbids it.
    ArithmeticWouldUnderflow,
    #[error("site finance arithmetic overflowed")]
    /// Checked revenue/labor arithmetic overflowed the supported money range.
    ArithmeticOverflow,
    #[error("site finance money value is invalid")]
    /// A money value failed semantic validation during finance calculation.
    InvalidMoney,
    #[error("site finance recommendation requires manager review")]
    /// Recommendation or action promotion was attempted without the manager-review gate.
    ManagerReviewRequired,
}

/// Result type for site-finance workflow validation and proof promotion.
pub type Result<T> = std::result::Result<T, Error>;
