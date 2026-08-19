use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Caller-reported feedback labels retained as nonclaimable evidence.
pub enum FeedbackOutcome {
    /// Retains a caller-reported completion label without proving completion.
    Completed,
    /// Retains a caller-reported deferral label without proving review.
    Deferred,
    /// Retains a caller-reported suppression label without proving manager action.
    SuppressedByManager,
    /// Retains a caller-reported wrong-source label for later reconciliation.
    SourceFactWasWrong,
    /// Retains a caller-reported not-actionable label without proving review.
    NotActionable,
}

impl FeedbackOutcome {}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Non-empty source-record references correlated to caller-reported outcome evidence.
pub struct OutcomeSourceRecordRefs(NonEmpty<source::RecordRef>);

impl OutcomeSourceRecordRefs {
    /// Builds non-empty source references without treating them as review or completion proof.
    pub fn try_new(source_record_refs: Vec<source::RecordRef>) -> Result<Self> {
        NonEmpty::from_vec(source_record_refs)
            .map(Self)
            .ok_or(Error::OutcomeSourceRecordRefRequired)
    }

    /// Returns source refs correlated to the report without proving review or live mutation.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &source::RecordRef> {
        self.0.iter()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Non-empty data-quality issue references correlated to caller-reported outcome evidence.
pub struct OutcomeIssueRefs(NonEmpty<IssueRef>);

impl OutcomeIssueRefs {
    /// Builds non-empty issue references without treating them as review or completion proof.
    pub fn try_new(issue_refs: Vec<IssueRef>) -> Result<Self> {
        NonEmpty::from_vec(issue_refs)
            .map(Self)
            .ok_or(Error::OutcomeIssueRefRequired)
    }

    /// Returns issue refs correlated to the report without proving review or live mutation.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &IssueRef> {
        self.0.iter()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Outcome record used by the data-quality hygiene workflow; it finds duplicate, stale, or inconsistent records while blocking automatic provider-system mutation.
pub struct OutcomeRecord {
    action_id: ActionId,
    recorded_by: entities::ActorRef,
    outcome: FeedbackOutcome,
    before_minutes: LaborMinutes,
    actual_minutes: LaborMinutes,
    source_record_refs: OutcomeSourceRecordRefs,
    issue_refs: OutcomeIssueRefs,
    reported_resolution_status: Option<data_quality::ResolutionStatus>,
}

impl std::fmt::Debug for OutcomeRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OutcomeRecord([REDACTED])")
    }
}

impl OutcomeRecord {
    /// Starts a reported-outcome builder that requires non-empty source and issue references.
    pub fn builder() -> OutcomeRecordBuilder {
        OutcomeRecordBuilder::default()
    }

    /// Returns the action id evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    /// Returns the recorded by evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn recorded_by(&self) -> &entities::ActorRef {
        &self.recorded_by
    }

    /// Returns the outcome evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn outcome(&self) -> FeedbackOutcome {
        self.outcome
    }

    /// Returns the before minutes evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn before_minutes(&self) -> LaborMinutes {
        self.before_minutes
    }

    /// Returns caller-reported minutes without claiming measured labor, review, or completion.
    pub const fn actual_minutes(&self) -> LaborMinutes {
        self.actual_minutes
    }

    /// Returns the source record refs evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn source_record_refs(&self) -> Vec<&source::RecordRef> {
        self.source_record_refs.iter().collect()
    }

    /// Returns the issue refs evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn issue_refs(&self) -> Vec<&IssueRef> {
        self.issue_refs.iter().collect()
    }

    /// Returns the caller-reported resolution-status label retained for later review.
    pub const fn reported_resolution_status(&self) -> Option<data_quality::ResolutionStatus> {
        self.reported_resolution_status
    }

    /// Returns the blocked actions evidence available to data-quality hygiene review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn blocked_actions(&self) -> Vec<BlockedAction> {
        blocked_actions_for()
    }
}

#[derive(Default, Clone)]
/// Builder for caller-reported outcome records; build requires non-empty source and issue refs.
pub struct OutcomeRecordBuilder {
    action_id: Option<ActionId>,
    recorded_by: Option<entities::ActorRef>,
    outcome: Option<FeedbackOutcome>,
    before_minutes: Option<LaborMinutes>,
    actual_minutes: Option<LaborMinutes>,
    source_record_refs: Vec<source::RecordRef>,
    issue_refs: Vec<IssueRef>,
    reported_resolution_status: Option<data_quality::ResolutionStatus>,
}

impl std::fmt::Debug for OutcomeRecordBuilder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OutcomeRecordBuilder([REDACTED])")
    }
}

impl OutcomeRecordBuilder {
    /// Sets the action-id correlation label for reported outcome evidence.
    pub fn action_id(mut self, action_id: ActionId) -> Self {
        self.action_id = Some(action_id);
        self
    }

    /// Sets the caller-provided actor label; this does not authenticate review.
    pub fn recorded_by(mut self, recorded_by: entities::ActorRef) -> Self {
        self.recorded_by = Some(recorded_by);
        self
    }

    /// Sets the caller-reported outcome label.
    pub fn outcome(mut self, outcome: FeedbackOutcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    /// Sets the estimated pre-cleanup labor minutes.
    pub fn before_minutes(mut self, before_minutes: LaborMinutes) -> Self {
        self.before_minutes = Some(before_minutes);
        self
    }

    /// Sets caller-reported minutes without proving cleanup or measured labor.
    pub fn actual_minutes(mut self, actual_minutes: LaborMinutes) -> Self {
        self.actual_minutes = Some(actual_minutes);
        self
    }

    /// Sets non-empty source refs correlated to reported outcome evidence.
    pub fn source_record_refs(mut self, source_record_refs: Vec<source::RecordRef>) -> Self {
        self.source_record_refs = source_record_refs;
        self
    }

    /// Sets non-empty issue refs correlated to reported outcome evidence.
    pub fn issue_refs(mut self, issue_refs: Vec<IssueRef>) -> Self {
        self.issue_refs = issue_refs;
        self
    }

    /// Sets the caller-reported resolution-status label.
    pub fn reported_resolution_status(
        mut self,
        reported_resolution_status: data_quality::ResolutionStatus,
    ) -> Self {
        self.reported_resolution_status = Some(reported_resolution_status);
        self
    }

    /// Builds the reported outcome record, rejecting absent source-record or issue refs.
    pub fn build(self) -> Result<OutcomeRecord> {
        Ok(OutcomeRecord {
            action_id: self
                .action_id
                .ok_or(Error::OutcomeFieldRequired("action_id"))?,
            recorded_by: self
                .recorded_by
                .ok_or(Error::OutcomeFieldRequired("recorded_by"))?,
            outcome: self.outcome.ok_or(Error::OutcomeFieldRequired("outcome"))?,
            before_minutes: self
                .before_minutes
                .ok_or(Error::OutcomeFieldRequired("before_minutes"))?,
            actual_minutes: self
                .actual_minutes
                .ok_or(Error::OutcomeFieldRequired("actual_minutes"))?,
            source_record_refs: OutcomeSourceRecordRefs::try_new(self.source_record_refs)?,
            issue_refs: OutcomeIssueRefs::try_new(self.issue_refs)?,
            reported_resolution_status: self.reported_resolution_status,
        })
    }
}
