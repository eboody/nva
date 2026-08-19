use super::*;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Stored caller-reported site-finance evidence and correlation labels.
///
/// This serializable row does not prove review, authorization, action, completion,
/// measured finance or labor, or value attribution.
pub struct SiteFinanceOutcomeRecord {
    /// Cross-system identifier tying projection, recommendation, action, and outcome together.
    pub correlation_id: String,
    /// Caller-reported location correlation; it does not prove review.
    pub location_id: String,
    /// Site finance reporting period start.
    pub period_start: String,
    /// Site finance reporting period end.
    pub period_end: String,
    /// Service line included in the site finance projection.
    pub service: String,
    /// Caller-reported recommendation correlation; it proves no reviewed workflow.
    pub recommendation_id: String,
    /// Caller-reported review-packet correlation; it grants no authorization.
    pub review_packet_id: String,
    /// Caller-reported audit-event correlation; it proves no review or action.
    pub audit_event_id: String,
    /// Safe action label; this must not become a payment, discount, refund, or accounting mutation.
    pub legal_action: String,
    /// Caller-reported value-attribution label; it supports no measured claim.
    pub value_attribution: SiteFinanceValueAttribution,
    /// Local workflow completion projection, separate from approval and value attribution.
    pub workflow_completion: SiteFinanceWorkflowCompletion,

    /// Currency code preserved for currency-aware reporting.
    pub currency: String,
    /// Net revenue in minor units after checked discount/refund arithmetic.
    pub net_revenue_minor_units: u64,
    /// Variance in minor units after relationship and currency checks.
    pub variance_minor_units: u64,
    #[builder(default)]
    /// Source evidence refs used to justify projection and outcome attribution.
    pub source_refs: Vec<StoredSourceRecordRef>,
    /// Caller-reported timestamp retained for historical correlation.
    pub recorded_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, Default)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Serializable evidence strength for a site-finance value-attribution candidate.
pub enum SiteFinanceValueAttribution {
    /// Historical evidence reports a reviewed action but cannot issue value-claim authority.
    #[default]
    ReportedReviewedAction,
    /// Correlated finance evidence is visible but cannot support a strong measured claim.
    CorrelatedOnly,
    /// Source evidence was wrong, so no measured value claim is allowed.
    WrongSource,
}

impl SiteFinanceValueAttribution {
    /// Returns whether serialized value-attribution evidence can support a strong value claim.
    ///
    /// It never does: a future authenticated boundary must issue opaque accepted attribution.
    pub const fn can_support_value_claim(self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, Default)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Caller-reported state for the local site-finance workflow projection.
///
/// These serializable labels are evidence only and cannot establish executable completion.
pub enum SiteFinanceWorkflowCompletion {
    /// Caller reports local completion as evidence only.
    #[default]
    ReportedCompleted,
    /// Workflow produced output that still needs review or follow-up.
    NeedsReview,
    /// Workflow was intentionally deferred.
    Deferred,
    /// Workflow was cancelled before local completion.
    Cancelled,
}

impl SiteFinanceWorkflowCompletion {
    pub(super) const fn workflow_result_status(self) -> WorkflowResultStatusCode {
        match self {
            Self::ReportedCompleted | Self::NeedsReview => WorkflowResultStatusCode::NeedsReview,
            Self::Deferred => WorkflowResultStatusCode::Deferred,
            Self::Cancelled => WorkflowResultStatusCode::Cancelled,
        }
    }
}

impl SiteFinanceOutcomeRecord {}
