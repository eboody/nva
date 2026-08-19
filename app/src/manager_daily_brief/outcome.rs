use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Caller-reported feedback labels retained as nonclaimable evidence.
pub enum FeedbackOutcome {
    /// Retains a caller-reported completion label without proving completion.
    Completed,
    /// Retains a caller-reported deferral label without proving review.
    Deferred,
    /// Retains a caller-reported suppression label without proving review.
    SuppressedByManager,
    /// Retains a caller-reported wrong-source label without proving review.
    SourceFactWasWrong,
}

impl FeedbackOutcome {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Normalized caller-reported disposition retained as nonclaimable outcome evidence.
pub enum ReportedDisposition {
    /// Caller supplied a completion label; no human action or completion is proven.
    CompletedLabel,
    /// Caller supplied a deferral label; no review, manager action, or deferral is proven.
    DeferredLabel,
    /// Caller supplied a suppression label; no review, manager action, or suppression is proven.
    SuppressedLabel,
    /// Caller supplied a wrong-source label; no source adjudication or rejection is proven.
    WrongSourceLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Reason an outcome is retained as feedback without counting optimistic labor savings.
pub enum LaborSavingsNotClaimedReason {
    /// Caller supplied a deferral label without proving review, manager action, or deferral.
    ReportedDeferredLabel,
    /// Caller supplied a suppression label without proving review, manager action, or suppression.
    ReportedSuppressedLabel,
    /// Caller supplied a wrong-source label without proving adjudication or rejection.
    ReportedWrongSourceLabel,
    /// Caller supplied a completion label without proving action, review, completion, or source provenance.
    ReportedCompletedLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Fail-closed labor-value claim disposition for serializable outcome history.
pub enum LaborSavingsClaim {
    /// Caller-reported feedback retained, but review and realized savings are not claimed.
    NotClaimed {
        /// Why the workflow preserves feedback without counting optimistic labor savings.
        reason: LaborSavingsNotClaimedReason,
    },
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Outcome record used by the manager daily brief workflow; it assembles reviewable manager brief packets from deterministic context and agent drafts.
pub struct OutcomeRecord {
    action_id: ActionId,
    recorded_by: entities::ActorRef,
    outcome: FeedbackOutcome,
    before_minutes: LaborMinutes,
    actual_minutes: LaborMinutes,
    manager_feedback: Option<ManagerFeedback>,
    #[builder(default)]
    source_record_refs: Vec<source::RecordRef>,
}

impl fmt::Debug for OutcomeRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OutcomeRecord([REDACTED])")
    }
}

impl OutcomeRecord {
    /// Returns the action id evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    /// Returns the recorded by evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn recorded_by(&self) -> &entities::ActorRef {
        &self.recorded_by
    }

    /// Returns the outcome evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn outcome(&self) -> FeedbackOutcome {
        self.outcome
    }

    /// Returns the before minutes evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn before_minutes(&self) -> LaborMinutes {
        self.before_minutes
    }

    /// Returns caller-reported minute evidence without proving measured labor, review, or completion.
    pub const fn actual_minutes(&self) -> LaborMinutes {
        self.actual_minutes
    }

    /// Returns the source record refs evidence available to manager daily brief review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn source_record_refs(&self) -> &[source::RecordRef] {
        &self.source_record_refs
    }

    /// Returns optional caller-reported feedback without authenticating a manager or reviewed disposition.
    pub const fn manager_feedback(&self) -> Option<&ManagerFeedback> {
        self.manager_feedback.as_ref()
    }
}
