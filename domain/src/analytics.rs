//! Analytics read models for resort operations after source validation.
//!
//! Analytics facts in this module sit after source ingestion and data-quality validation:
//! raw Gingr/provider records are preserved as provenance, blocking hygiene findings stop
//! projection, and nonblocking findings stay attached so manager briefs and labor-cost
//! dashboards can explain their evidence instead of inventing operational truth.

use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::operations::labor;
use crate::{entities, money, policy, source};

/// Source-backed financial facts and review-gated site finance insights.
///
/// Canonical owner for finance concepts previously introduced by `strategic_ai_ops::financial`.
/// Insights cannot mutate price, discount, refund, payment, or accounting state.
pub mod finance {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    /// Reporting period for one site.
    pub struct SitePeriod {
        location_id: entities::LocationId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    }

    impl SitePeriod {
        /// Starts a manual builder that validates start/end order.
        pub const fn builder() -> SitePeriodBuilder {
            SitePeriodBuilder::new()
        }
    }

    impl<'de> Deserialize<'de> for SitePeriod {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct RawSitePeriod {
                location_id: entities::LocationId,
                start: DateTime<Utc>,
                end: DateTime<Utc>,
            }

            let raw = RawSitePeriod::deserialize(deserializer)?;
            Self::builder()
                .location_id(raw.location_id)
                .start(raw.start)
                .end(raw.end)
                .build()
                .map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    /// Builder for a site financial reporting period.
    pub struct SitePeriodBuilder {
        location_id: Option<entities::LocationId>,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    }

    impl SitePeriodBuilder {
        /// Creates an empty builder.
        pub const fn new() -> Self {
            Self {
                location_id: None,
                start: None,
                end: None,
            }
        }

        /// Sets location id.
        pub const fn location_id(mut self, value: entities::LocationId) -> Self {
            self.location_id = Some(value);
            self
        }

        /// Sets period start.
        pub const fn start(mut self, value: DateTime<Utc>) -> Self {
            self.start = Some(value);
            self
        }

        /// Sets period end.
        pub const fn end(mut self, value: DateTime<Utc>) -> Self {
            self.end = Some(value);
            self
        }

        /// Builds a period if all required values exist and end follows start.
        pub fn build(self) -> std::result::Result<SitePeriod, Error> {
            let location_id = self.location_id.ok_or(Error::MissingLocation)?;
            let start = self.start.ok_or(Error::MissingStart)?;
            let end = self.end.ok_or(Error::MissingEnd)?;
            if end <= start {
                return Err(Error::EndMustFollowStart);
            }
            Ok(SitePeriod {
                location_id,
                start,
                end,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Source-backed site/service financial fact.
    pub struct ReportedRevenueObservationFact {
        period: SitePeriod,
        service: entities::ServiceKind,
        gross_revenue: money::Money,
        discount: money::Money,
        refund: money::Money,
        labor_cost: money::Money,
        source_system: source::System,
    }

    impl ReportedRevenueObservationFact {
        /// Net revenue after discounts and refunds, with checked currency-aware arithmetic.
        pub fn net_revenue(&self) -> money::Result<money::Money> {
            self.gross_revenue
                .checked_sub(self.discount.clone())?
                .checked_sub(self.refund.clone())
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Site financial insight kind.
    pub enum InsightKind {
        /// Discount leakage appears elevated.
        DiscountLeakage,
        /// Refund rate appears elevated.
        RefundRateVariance,
        /// Labor percent appears elevated.
        LaborCostVariance,
        /// Add-on attach rate opportunity.
        AddOnAttachRateOpportunity,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Review-gated financial recommendation.
    pub enum Recommendation {
        /// Review discount policy or approvals.
        ReviewDiscountPolicy,
        /// Review labor plan.
        ReviewLaborPlan,
        /// Review add-on attach process.
        ReviewAddOnWorkflow,
        /// Investigate source/accounting variance.
        InvestigateVariance,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    /// Review-gated site financial insight.
    pub struct Insight {
        fact: ReportedRevenueObservationFact,
        kind: InsightKind,
        expected_impact: money::Money,
        recommendation: Recommendation,
        review_gate: policy::ReviewGate,
    }

    impl Insight {
        /// Creates a financial insight only when the recommendation remains behind manager review.
        pub fn try_new(
            fact: ReportedRevenueObservationFact,
            kind: InsightKind,
            expected_impact: money::Money,
            recommendation: Recommendation,
            review_gate: policy::ReviewGate,
        ) -> std::result::Result<Self, Error> {
            if review_gate != policy::ReviewGate::ManagerApproval {
                return Err(Error::ManagerReviewRequired);
            }
            Ok(Self {
                fact,
                kind,
                expected_impact,
                recommendation,
                review_gate,
            })
        }

        /// Starts an insight builder that validates the manager-review gate at build time.
        pub const fn builder() -> InsightBuilder {
            InsightBuilder::new()
        }

        /// Net revenue on the underlying fact.
        pub fn net_revenue(&self) -> money::Result<money::Money> {
            self.fact.net_revenue()
        }

        /// Financial insight cannot directly mutate price, discount, payment, or accounting state.
        pub const fn blocks_financial_mutation(&self) -> bool {
            true
        }
    }

    impl<'de> Deserialize<'de> for Insight {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct RawInsight {
                fact: ReportedRevenueObservationFact,
                kind: InsightKind,
                expected_impact: money::Money,
                recommendation: Recommendation,
                review_gate: policy::ReviewGate,
            }

            let raw = RawInsight::deserialize(deserializer)?;
            Self::try_new(
                raw.fact,
                raw.kind,
                raw.expected_impact,
                raw.recommendation,
                raw.review_gate,
            )
            .map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Default)]
    /// Builder for a review-gated site financial insight.
    pub struct InsightBuilder {
        fact: Option<ReportedRevenueObservationFact>,
        kind: Option<InsightKind>,
        expected_impact: Option<money::Money>,
        recommendation: Option<Recommendation>,
        review_gate: Option<policy::ReviewGate>,
    }

    impl InsightBuilder {
        /// Creates an empty builder.
        pub const fn new() -> Self {
            Self {
                fact: None,
                kind: None,
                expected_impact: None,
                recommendation: None,
                review_gate: None,
            }
        }

        /// Sets the financial fact.
        pub fn fact(mut self, value: ReportedRevenueObservationFact) -> Self {
            self.fact = Some(value);
            self
        }

        /// Sets the insight kind.
        pub const fn kind(mut self, value: InsightKind) -> Self {
            self.kind = Some(value);
            self
        }

        /// Sets the expected impact.
        pub fn expected_impact(mut self, value: money::Money) -> Self {
            self.expected_impact = Some(value);
            self
        }

        /// Sets the recommendation.
        pub const fn recommendation(mut self, value: Recommendation) -> Self {
            self.recommendation = Some(value);
            self
        }

        /// Sets the review gate.
        pub const fn review_gate(mut self, value: policy::ReviewGate) -> Self {
            self.review_gate = Some(value);
            self
        }

        /// Builds the insight if every required field is present and manager-reviewed.
        pub fn build(self) -> std::result::Result<Insight, Error> {
            Insight::try_new(
                self.fact.ok_or(Error::MissingFact)?,
                self.kind.ok_or(Error::MissingKind)?,
                self.expected_impact.ok_or(Error::MissingExpectedImpact)?,
                self.recommendation.ok_or(Error::MissingRecommendation)?,
                self.review_gate.ok_or(Error::MissingReviewGate)?,
            )
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Site-period validation failure.
    pub enum Error {
        #[error("financial site period requires a location")]
        /// Location id missing.
        MissingLocation,
        #[error("financial site period requires a start")]
        /// Start missing.
        MissingStart,
        #[error("financial site period requires an end")]
        /// End missing.
        MissingEnd,
        #[error("financial site period end must follow start")]
        /// End did not follow start.
        EndMustFollowStart,
        #[error("financial insight requires manager review")]
        /// Financial recommendations cannot use customer-message or other non-manager gates.
        ManagerReviewRequired,
        #[error("financial insight requires a fact")]
        /// Fact missing from financial insight builder.
        MissingFact,
        #[error("financial insight requires a kind")]
        /// Kind missing from financial insight builder.
        MissingKind,
        #[error("financial insight requires expected impact")]
        /// Expected impact missing from financial insight builder.
        MissingExpectedImpact,
        #[error("financial insight requires a recommendation")]
        /// Recommendation missing from financial insight builder.
        MissingRecommendation,
        #[error("financial insight requires a review gate")]
        /// Review gate missing from financial insight builder.
        MissingReviewGate,
    }
}

/// Outcome attribution records used before making labor, revenue, or conversion value claims.
///
/// Canonical owner for outcome concepts previously introduced by `strategic_ai_ops::outcome`.
/// Value claims require reviewed-action attribution and source evidence.
pub mod outcome {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
        derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize
        )
    )]
    /// Outcome record id.
    pub struct Id(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
        derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize
        )
    )]
    /// Reviewed recommendation identifier linked to an observed outcome.
    pub struct RecommendationRef(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
        derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize
        )
    )]
    /// Source or review evidence identifier used for outcome attribution.
    pub struct EvidenceRef(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Strategic AI-ops workstream.
    pub enum Workstream {
        /// Real-time lead response.
        LeadResponse,
        /// Capacity/labor optimization.
        CapacityLabor,
        /// Knowledge assistant.
        KnowledgeAssistant,
        /// Retention.
        Retention,
        /// Site financial insights.
        FinancialInsights,
        /// CRM note intelligence.
        CrmIntelligence,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Caller-reported metric-evidence category; never value-claim authority.
    pub enum Metric {
        /// Reported booking-conversion observation.
        ReportedBookingConversionObservation,
        /// Reported labor-minute estimate difference; not realized savings.
        ReportedLaborMinutesDifference,
        /// Reported utilization basis-points observation.
        ReportedUtilizationBasisPointsObservation,
        /// Reported revenue observation using canonical currency-aware money.
        ReportedRevenueObservation,
        /// Legacy reported revenue-cents observation.
        ReportedRevenueCentsObservation,
        /// Reported customer-retention observation.
        ReportedCustomerRetentionObservation,
        /// Reported handle-time estimate difference.
        ReportedHandleTimeEstimateDifference,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Metric value representation retained for legacy bridge compatibility.
    pub enum MetricValue {
        /// Count value.
        Count(i64),
        /// Positive labor-minute value.
        LaborMinutes(labor::Minutes),
        /// Reported money evidence with explicit currency.
        ReportedRevenueObservation(money::Money),
        /// Basis-points value.
        ReportedUtilizationBasisPointsObservation(money::BasisPoints),
        /// Legacy minutes value retained for bridge compatibility.
        Minutes(i64),
        /// Legacy cents value retained for bridge compatibility.
        Cents(i64),
        /// Legacy basis-points value retained for bridge compatibility.
        BasisPoints(i64),
    }

    impl MetricValue {
        /// Builds a reported labor-minute estimate-difference value.
        pub const fn reported_labor_minutes_difference(value: labor::Minutes) -> Self {
            Self::LaborMinutes(value)
        }

        /// Builds a currency-aware revenue metric value.
        pub fn revenue(value: money::Money) -> Self {
            Self::ReportedRevenueObservation(value)
        }

        /// Builds a utilization metric value in validated basis points.
        pub const fn utilization_basis_points(value: money::BasisPoints) -> Self {
            Self::ReportedUtilizationBasisPointsObservation(value)
        }

        /// Returns whether this value carries the unit required by the metric.
        pub const fn matches_metric(&self, metric: Metric) -> bool {
            matches!(
                (metric, self),
                (
                    Metric::ReportedBookingConversionObservation
                        | Metric::ReportedCustomerRetentionObservation,
                    Self::Count(_)
                ) | (
                    Metric::ReportedLaborMinutesDifference,
                    Self::LaborMinutes(_)
                ) | (Metric::ReportedLaborMinutesDifference, Self::Minutes(_))
                    | (
                        Metric::ReportedHandleTimeEstimateDifference,
                        Self::LaborMinutes(_)
                    )
                    | (
                        Metric::ReportedHandleTimeEstimateDifference,
                        Self::Minutes(_)
                    )
                    | (
                        Metric::ReportedRevenueObservation,
                        Self::ReportedRevenueObservation(_)
                    )
                    | (
                        Metric::ReportedRevenueCentsObservation,
                        Self::ReportedRevenueObservation(_)
                    )
                    | (Metric::ReportedRevenueCentsObservation, Self::Cents(_))
                    | (
                        Metric::ReportedUtilizationBasisPointsObservation,
                        Self::ReportedUtilizationBasisPointsObservation(_)
                    )
                    | (
                        Metric::ReportedUtilizationBasisPointsObservation,
                        Self::BasisPoints(_)
                    )
            )
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Caller-reported before/after evidence whose variants own the metric-compatible unit.
    pub enum MeasuredChange {
        /// Caller-reported booking-conversion count observation.
        ReportedBookingConversionObservation {
            /// Caller-reported baseline conversion count.
            before: i64,
            /// Caller-reported comparison conversion count.
            after: i64,
        },
        /// Reported labor-minute estimate difference.
        ReportedLaborMinutesDifference {
            /// Caller-reported baseline labor-minute estimate.
            before: labor::Minutes,
            /// Caller-reported comparison labor-minute estimate.
            after: labor::Minutes,
        },
        /// Caller-reported utilization basis-points observation.
        ReportedUtilizationBasisPointsObservation {
            /// Caller-reported baseline utilization value.
            before: money::BasisPoints,
            /// Caller-reported comparison utilization value.
            after: money::BasisPoints,
        },
        /// Reported revenue observation using canonical currency-aware money.
        ReportedRevenueObservation {
            /// Caller-reported baseline revenue observation.
            before: money::Money,
            /// Caller-reported comparison revenue observation.
            after: money::Money,
        },
        /// Caller-reported customer-retention count observation.
        ReportedCustomerRetentionObservation {
            /// Caller-reported baseline retained-customer count.
            before: i64,
            /// Caller-reported comparison retained-customer count.
            after: i64,
        },
        /// Caller-reported handle-time estimate difference.
        ReportedHandleTimeEstimateDifference {
            /// Caller-reported baseline handle-time estimate.
            before: labor::Minutes,
            /// Caller-reported comparison handle-time estimate.
            after: labor::Minutes,
        },
    }

    impl MeasuredChange {
        /// Builds a booking-conversion count change.
        pub const fn booking_converted(before: i64, after: i64) -> Self {
            Self::ReportedBookingConversionObservation { before, after }
        }

        /// Builds a currency-aware revenue change.
        pub fn revenue(before: money::Money, after: money::Money) -> Self {
            Self::ReportedRevenueObservation { before, after }
        }

        /// Metric label carried by this caller-reported observation.
        pub const fn metric(&self) -> Metric {
            match self {
                Self::ReportedBookingConversionObservation { .. } => {
                    Metric::ReportedBookingConversionObservation
                }
                Self::ReportedLaborMinutesDifference { .. } => {
                    Metric::ReportedLaborMinutesDifference
                }
                Self::ReportedUtilizationBasisPointsObservation { .. } => {
                    Metric::ReportedUtilizationBasisPointsObservation
                }
                Self::ReportedRevenueObservation { .. } => Metric::ReportedRevenueObservation,
                Self::ReportedCustomerRetentionObservation { .. } => {
                    Metric::ReportedCustomerRetentionObservation
                }
                Self::ReportedHandleTimeEstimateDifference { .. } => {
                    Metric::ReportedHandleTimeEstimateDifference
                }
            }
        }

        /// Returns whether before/after values share required relationship-level context.
        pub fn relationship_context_is_consistent(&self) -> bool {
            match self {
                Self::ReportedRevenueObservation { before, after } => {
                    before.currency() == after.currency()
                }
                _ => true,
            }
        }

        fn try_from_metric_values(
            metric: Metric,
            before_value: MetricValue,
            after_value: MetricValue,
        ) -> Result<Self> {
            match (metric, before_value, after_value) {
                (
                    Metric::ReportedBookingConversionObservation,
                    MetricValue::Count(before),
                    MetricValue::Count(after),
                ) => Ok(Self::ReportedBookingConversionObservation { before, after }),
                (
                    Metric::ReportedCustomerRetentionObservation,
                    MetricValue::Count(before),
                    MetricValue::Count(after),
                ) => Ok(Self::ReportedCustomerRetentionObservation { before, after }),
                (
                    Metric::ReportedLaborMinutesDifference,
                    MetricValue::LaborMinutes(before),
                    MetricValue::LaborMinutes(after),
                ) => Ok(Self::ReportedLaborMinutesDifference { before, after }),
                (
                    Metric::ReportedHandleTimeEstimateDifference,
                    MetricValue::LaborMinutes(before),
                    MetricValue::LaborMinutes(after),
                ) => Ok(Self::ReportedHandleTimeEstimateDifference { before, after }),
                (
                    Metric::ReportedUtilizationBasisPointsObservation,
                    MetricValue::ReportedUtilizationBasisPointsObservation(before),
                    MetricValue::ReportedUtilizationBasisPointsObservation(after),
                ) => Ok(Self::ReportedUtilizationBasisPointsObservation { before, after }),
                (
                    Metric::ReportedRevenueObservation,
                    MetricValue::ReportedRevenueObservation(before),
                    MetricValue::ReportedRevenueObservation(after),
                )
                | (
                    Metric::ReportedRevenueCentsObservation,
                    MetricValue::ReportedRevenueObservation(before),
                    MetricValue::ReportedRevenueObservation(after),
                ) => Ok(Self::ReportedRevenueObservation { before, after }),
                _ => Err(Error::MetricValueUnitMismatch),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Legacy attribution strength for compatibility constructors.
    pub enum Attribution {
        /// Serializable evidence carries a reviewed-action label; no human review is proven.
        ReportedReviewedAction,
        /// Correlated with recommendation but not enough for strong claims.
        CorrelatedOnly,
        /// Source was wrong, so no value claim is allowed.
        WrongSource,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Evidence bundle that explains attribution sufficiency.
    pub enum AttributionEvidence {
        /// Serializable evidence carries a reviewed-action label but proves no review or value.
        ReportedReviewedAction {
            /// Caller-reported recommendation or action identifier.
            recommendation_ref: RecommendationRef,
            /// Caller-reported source/evidence identifier.
            evidence_ref: EvidenceRef,
        },
        /// Correlated evidence is visible but not sufficient for strong claims.
        CorrelatedOnly {
            /// Correlation evidence identifier.
            evidence_ref: EvidenceRef,
        },
        /// Wrong-source evidence prevents claims.
        WrongSource {
            /// Wrong-source evidence identifier.
            evidence_ref: EvidenceRef,
        },
    }

    impl AttributionEvidence {
        /// Builds historical reported-reviewed compatibility evidence without proving review.
        pub const fn reported_reviewed_action(
            recommendation_ref: RecommendationRef,
            evidence_ref: EvidenceRef,
        ) -> Self {
            Self::ReportedReviewedAction {
                recommendation_ref,
                evidence_ref,
            }
        }

        /// Builds weak correlated attribution that cannot support a value claim.
        pub const fn correlated_only(evidence_ref: EvidenceRef) -> Self {
            Self::CorrelatedOnly { evidence_ref }
        }

        /// Returns whether serialized evidence suffices for a measured value claim.
        ///
        /// It never does: accepted value attribution requires a future opaque, authenticated issuer.
        pub const fn can_support_value_claim(&self) -> bool {
            false
        }

        fn from_legacy(attribution: Attribution) -> Self {
            let evidence_ref = EvidenceRef::try_new("legacy-outcome-attribution")
                .expect("static legacy outcome evidence ref is valid");
            match attribution {
                Attribution::ReportedReviewedAction => Self::ReportedReviewedAction {
                    recommendation_ref: RecommendationRef::try_new("legacy-reviewed-action")
                        .expect("static legacy recommendation ref is valid"),
                    evidence_ref,
                },
                Attribution::CorrelatedOnly => Self::CorrelatedOnly { evidence_ref },
                Attribution::WrongSource => Self::WrongSource { evidence_ref },
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Caller-reported outcome observation retained without review or business-value authority.
    pub struct Record {
        id: Id,
        workstream: Workstream,
        location_id: entities::LocationId,
        change: MeasuredChange,
        attribution: AttributionEvidence,
        source: source::System,
        recorded_at: DateTime<Utc>,
    }

    impl Record {
        /// Creates an outcome record only when both metric values match the declared metric unit.
        #[allow(clippy::too_many_arguments)]
        pub fn try_new(
            id: Id,
            workstream: Workstream,
            location_id: entities::LocationId,
            metric: Metric,
            before_value: MetricValue,
            after_value: MetricValue,
            attribution: Attribution,
            source: source::System,
            recorded_at: DateTime<Utc>,
        ) -> Result<Self> {
            if !before_value.matches_metric(metric) || !after_value.matches_metric(metric) {
                return Err(Error::MetricValueUnitMismatch);
            }
            Ok(Self {
                id,
                workstream,
                location_id,
                change: MeasuredChange::try_from_metric_values(metric, before_value, after_value)?,
                attribution: AttributionEvidence::from_legacy(attribution),
                source,
                recorded_at,
            })
        }

        /// Workstream this outcome belongs to.
        pub const fn workstream(&self) -> Workstream {
            self.workstream
        }

        /// Metric label associated with this caller-reported before/after observation.
        pub const fn metric(&self) -> Metric {
            self.change.metric()
        }

        /// Returns false because caller-reported observations cannot support a value claim.
        pub fn can_support_value_claim(&self) -> bool {
            self.attribution.can_support_value_claim()
                && self.change.relationship_context_is_consistent()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Outcome validation failures.
    pub enum Error {
        #[error("outcome metric value units must match the declared metric")]
        /// Before/after values carried units incompatible with the metric.
        MetricValueUnitMismatch,
    }

    /// Result type returned by outcome constructors.
    pub type Result<T> = std::result::Result<T, Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Version tag for a deterministic analytics projection.
///
/// This is the read-model side of the source-fact → validated-domain → workflow chain:
/// it records which projection logic turned provider reservations into labor, demand,
/// and manager-brief evidence so downstream reports can compare like with like.
pub struct ProjectionVersion(String);

impl ProjectionVersion {
    /// Validates the projection-version label before reports rely on it for comparisons.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyProjectionVersion).map(Self)
    }

    /// Returns the projection-version identifier for storage/read-model boundaries.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ProjectionVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Projected stay facts for reservation records that passed source validation.
pub mod stay {
    use serde::{Deserialize, Serialize};

    use crate::{analytics, data_quality, source};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Stable analytics id for a projected stay fact, distinct from provider record ids.
    pub struct Id(String);

    impl Id {
        /// Builds a projected-stay analytics id so reports do not reuse raw provider record ids.
        pub fn try_new(value: impl Into<String>) -> analytics::Result<Self> {
            analytics::trimmed_non_empty(value, analytics::Error::EmptyStayFactId).map(Self)
        }

        /// Returns the provider or domain identifier as a string slice.
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl<'de> Deserialize<'de> for Id {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Whether a stay projection is clean, reviewable, or blocked by source hygiene.
    pub enum DataQualityStatus {
        /// Source facts validated cleanly and can feed labor/read-model workflows directly.
        Complete,
        /// Projection is usable, but nonblocking hygiene issues should be visible to managers.
        ManagerReviewRequired,
        /// Source facts are not safe enough to power workflow or labor-cost decisions.
        BlockingIssues,
    }

    #[derive(Clone, PartialEq, Eq, Serialize)]
    /// Projected stay fact used by analytics, manager briefs, and labor planning.
    pub struct Fact {
        id: Id,
        provenance: source::Provenance,
        reservation_record_id: source::record::Id,
        customer_record_id: source::record::Id,
        pet_record_id: source::record::Id,
        location_record_id: source::record::Id,
        service_type_record_id: source::record::Id,
        projection_version: analytics::ProjectionVersion,
        data_quality_status: DataQualityStatus,
        data_quality_issues: Vec<data_quality::Issue>,
    }

    impl std::fmt::Debug for Fact {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("stay::Fact([REDACTED])")
        }
    }

    impl Fact {
        /// Projects a validated stay fact from a source reservation snapshot.
        ///
        /// Blocking data-quality issues return the full issue set instead of producing a
        /// fact; nonblocking issues stay attached as evidence for reviewable read models.
        pub fn project_from_source_reservation(
            id: Id,
            source_reservation: &source::reservation::Snapshot,
            projection_version: analytics::ProjectionVersion,
        ) -> std::result::Result<Self, Vec<data_quality::Issue>> {
            let issues = source_reservation
                .data_quality_issues(source_reservation.provenance().pulled_at().clone());
            if issues.iter().any(data_quality::Issue::workflow_blocking) {
                return Err(issues);
            }
            let data_quality_status = if issues.is_empty() {
                DataQualityStatus::Complete
            } else {
                DataQualityStatus::ManagerReviewRequired
            };

            let customer_record_id = source_reservation
                .customer_record_id()
                .expect("data_quality_issues guards customer presence")
                .clone();
            let pet_record_id = source_reservation
                .pet_record_id()
                .expect("data_quality_issues guards pet presence")
                .clone();
            let location_record_id = source_reservation
                .location_record_id()
                .expect("data_quality_issues guards location presence")
                .clone();
            let service_type_record_id = source_reservation
                .service_type_record_id()
                .expect("data_quality_issues guards service type presence")
                .clone();

            Ok(Self {
                id,
                provenance: source_reservation.provenance().clone(),
                reservation_record_id: source_reservation.provenance().record_id().clone(),
                customer_record_id,
                pet_record_id,
                location_record_id,
                service_type_record_id,
                projection_version,
                data_quality_status,
                data_quality_issues: issues,
            })
        }

        /// Returns the analytics fact id used to join reports without reusing provider record ids.
        pub const fn id(&self) -> &Id {
            &self.id
        }

        /// Returns the provider system that supplied the stay evidence.
        pub const fn source_system(&self) -> source::System {
            self.provenance.source_system()
        }

        /// Returns the source provenance managers can inspect before trusting a brief or labor report.
        pub const fn provenance(&self) -> &source::Provenance {
            &self.provenance
        }

        /// Returns the source reservation record that explains the projected stay.
        pub const fn reservation_record_id(&self) -> &source::record::Id {
            &self.reservation_record_id
        }

        /// Returns the source customer record needed for reviewed communication or cleanup workflows.
        pub const fn customer_record_id(&self) -> &source::record::Id {
            &self.customer_record_id
        }

        /// Returns the source pet record needed for care, eligibility, and safety review.
        pub const fn pet_record_id(&self) -> &source::record::Id {
            &self.pet_record_id
        }

        /// Returns the source location record used to keep demand tied to the correct resort.
        pub const fn location_record_id(&self) -> &source::record::Id {
            &self.location_record_id
        }

        /// Returns the source service-type record used before demand is grouped by service line.
        pub const fn service_type_record_id(&self) -> &source::record::Id {
            &self.service_type_record_id
        }

        /// Returns the projection version so dashboards compare facts produced by the same logic.
        pub const fn projection_version(&self) -> &analytics::ProjectionVersion {
            &self.projection_version
        }

        /// Returns whether the fact is clean or still needs manager-visible data-quality review.
        pub const fn data_quality_status(&self) -> DataQualityStatus {
            self.data_quality_status
        }

        /// Nonblocking source data-quality issues preserved on the projected stay fact.
        ///
        /// Workflow-blocking issues are returned as projection errors instead of producing a fact.
        pub fn data_quality_issues(&self) -> &[data_quality::Issue] {
            &self.data_quality_issues
        }
    }
}

/// Aggregated service-demand facts used to compare booked work against labor capacity.
pub mod service_demand {
    use serde::{Deserialize, Serialize};

    use crate::{analytics, data_quality, operations, source};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Provider or source identifier retained as the stable join key.
    pub struct Id(String);

    impl Id {
        /// Builds a service-demand fact only when source records prove the booked work being counted.
        pub fn try_new(value: impl Into<String>) -> analytics::Result<Self> {
            analytics::trimmed_non_empty(value, analytics::Error::EmptyServiceDemandFactId)
                .map(Self)
        }

        /// Returns the provider or domain identifier as a string slice.
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl<'de> Deserialize<'de> for Id {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Nonzero count of work units for a service line on an operating day.
    pub struct DemandUnits(u32);

    impl DemandUnits {
        /// Accepts only nonzero demand units so labor reports cannot hide real booked work.
        pub const fn try_new(value: u32) -> analytics::Result<Self> {
            if value == 0 {
                return Err(analytics::Error::EmptyDemandUnits);
            }
            Ok(Self(value))
        }

        /// Returns the nonzero work-unit count for reports, storage rows, and adapter payloads.
        pub const fn get(self) -> u32 {
            self.0
        }
    }

    impl<'de> Deserialize<'de> for DemandUnits {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = u32::deserialize(deserializer)?;
            Self::try_new(value).map_err(serde::de::Error::custom)
        }
    }

    #[derive(Clone, PartialEq, Eq, Serialize)]
    /// Source-backed service-demand fact for labor planning and exception reporting.
    pub struct Fact {
        id: Id,
        operating_day: operations::operating_day::Key,
        demand_units: DemandUnits,
        source_record_refs: Vec<source::RecordRef>,
        projection_version: analytics::ProjectionVersion,
        data_quality_status: DataQualityStatus,
        data_quality_issues: Vec<data_quality::Issue>,
    }

    impl std::fmt::Debug for Fact {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("service_demand::Fact([REDACTED])")
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Analytics data-quality state that controls whether service-demand facts can appear as clean or reviewable.
    pub enum DataQualityStatus {
        /// Demand fact has required source evidence and no attached hygiene findings.
        Complete,
        /// Demand fact can inform reports, but attached hygiene findings must stay visible to managers.
        ManagerReviewRequired,
    }

    impl Fact {
        /// Builds a service-demand fact only when source records prove the booked work being counted.
        pub fn try_new(
            id: Id,
            operating_day: operations::operating_day::Key,
            demand_units: DemandUnits,
            source_record_refs: Vec<source::RecordRef>,
            projection_version: analytics::ProjectionVersion,
            data_quality_issues: Vec<data_quality::Issue>,
        ) -> Result<Self> {
            if source_record_refs.is_empty() {
                return Err(Error::MissingSourceEvidence);
            }
            let data_quality_status = if data_quality_issues.is_empty() {
                DataQualityStatus::Complete
            } else {
                DataQualityStatus::ManagerReviewRequired
            };

            Ok(Self {
                id,
                operating_day,
                demand_units,
                source_record_refs,
                projection_version,
                data_quality_status,
                data_quality_issues,
            })
        }

        /// Returns the analytics fact id used to join reports without reusing provider record ids.
        pub const fn id(&self) -> &Id {
            &self.id
        }

        /// Returns the resort/service/day bucket used to compare demand with staffing and capacity.
        pub const fn operating_day(&self) -> &operations::operating_day::Key {
            &self.operating_day
        }

        /// Returns the booked work units that drive labor planning and exception ranking.
        pub const fn demand_units(&self) -> DemandUnits {
            self.demand_units
        }

        /// Returns the source records that justify the demand units before labor reports rely on them.
        pub fn source_record_refs(&self) -> &[source::RecordRef] {
            &self.source_record_refs
        }

        /// Returns the projection version so dashboards compare facts produced by the same logic.
        pub const fn projection_version(&self) -> &analytics::ProjectionVersion {
            &self.projection_version
        }

        /// Returns whether the fact is clean or still needs manager-visible data-quality review.
        pub const fn data_quality_status(&self) -> DataQualityStatus {
            self.data_quality_status
        }

        /// Returns nonblocking hygiene findings that explain why demand evidence may need manager review.
        pub fn data_quality_issues(&self) -> &[data_quality::Issue] {
            &self.data_quality_issues
        }
    }

    impl<'de> Deserialize<'de> for Fact {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct RawFact {
                id: Id,
                operating_day: operations::operating_day::Key,
                demand_units: DemandUnits,
                source_record_refs: Vec<source::RecordRef>,
                projection_version: analytics::ProjectionVersion,
                data_quality_issues: Vec<data_quality::Issue>,
            }

            let raw = RawFact::deserialize(deserializer)?;
            Self::try_new(
                raw.id,
                raw.operating_day,
                raw.demand_units,
                raw.source_record_refs,
                raw.projection_version,
                raw.data_quality_issues,
            )
            .map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Validation failures returned by analytics domain constructors.
    pub enum Error {
        #[error("service demand facts require source evidence")]
        /// A demand metric was attempted without source records to prove the underlying work.
        MissingSourceEvidence,
    }

    /// Result type returned by fallible analytics operations.
    pub type Result<T> = std::result::Result<T, Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by analytics domain constructors.
pub enum Error {
    #[error("stay fact id must not be empty")]
    /// Signals that stay fact id was blank or missing during analytics validation.
    EmptyStayFactId,
    #[error("service demand fact id must not be empty")]
    /// Signals that service demand fact id was blank or missing during analytics validation.
    EmptyServiceDemandFactId,
    #[error("service demand units must be greater than zero")]
    /// Signals that demand units was blank or missing during analytics validation.
    EmptyDemandUnits,
    #[error("projection version must not be empty")]
    /// Signals that projection version was blank or missing during analytics validation.
    EmptyProjectionVersion,
}

/// Result type returned by fallible analytics operations.
pub type Result<T> = std::result::Result<T, Error>;

fn trimmed_non_empty(value: impl Into<String>, empty_error: Error) -> Result<String> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(empty_error);
    }
    Ok(value)
}
