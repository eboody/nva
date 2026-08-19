use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Groomer-assignment policies used when booking grooming work.
pub enum Policy {
    /// Staff can see the pay per session training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    PayPerSession,
    /// Sessions used by staff to prepare training assignment, package, progress, or parent-summary review.
    MultiSessionPackage {
        /// Session count that sets the purchased or reusable package balance.
        sessions: SessionCount,
    },
    /// Staff can see the board and train bundle training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    BoardAndTrainBundle,
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
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
pub struct Id(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Training package ledger event for purchases, reservations, consumption, and releases.
pub enum LedgerEntry {
    /// Sessions used by staff to prepare training assignment, package, progress, or parent-summary review.
    Purchased {
        /// Session count that sets the purchased or reusable package balance.
        sessions: SessionCount,
    },
    /// Session identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    Reserved {
        /// Training session tied to the package ledger or follow-up trigger.
        session_id: SessionId,
    },
    /// Session identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    Consumed {
        /// Training session tied to the package ledger or follow-up trigger.
        session_id: SessionId,
    },
    /// Session identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    Released {
        /// Training session tied to the package ledger or follow-up trigger.
        session_id: SessionId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Opening package ledger assembled from purchased, reserved, consumed, and released session facts.
pub struct OpeningLedger {
    /// Package identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub package_id: Id,
    /// Customer identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub customer_id: CustomerId,
    /// Pet receiving the training service or parent-facing progress update.
    pub pet_id: PetId,
    /// Policy used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub policy: Policy,
    /// Entries used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub entries: Vec<LedgerEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Training package ledger used to compute remaining reusable sessions without raw counters.
pub struct Ledger {
    package_id: Id,
    /// Customer identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub customer_id: CustomerId,
    /// Pet receiving the training service or parent-facing progress update.
    pub pet_id: PetId,
    policy: Policy,
    entries: Vec<LedgerEntry>,
}

impl<'de> Deserialize<'de> for Ledger {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opening = OpeningLedger::deserialize(deserializer)?;
        Self::open(opening).map_err(serde::de::Error::custom)
    }
}

impl Ledger {
    /// Opens a reusable package ledger after confirming the package policy has a session balance.
    pub fn open(opening: OpeningLedger) -> Result<Self> {
        if !matches!(opening.policy, Policy::MultiSessionPackage { .. }) {
            return Err(Error::PackageHasNoReusableBalance);
        }
        Ok(Self {
            package_id: opening.package_id,
            customer_id: opening.customer_id,
            pet_id: opening.pet_id,
            policy: opening.policy,
            entries: opening.entries,
        })
    }
    /// Returns the package id value used by training assignment, progress, package, or parent-summary review.
    pub fn package_id(&self) -> &Id {
        &self.package_id
    }
    /// Returns the entries value used by training assignment, progress, package, or parent-summary review.
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Package usage decision for reserving the next session or escalating balance/reconciliation issues.
pub enum UsageDecision {
    /// Staff can see the reserve next session training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    ReserveNextSession {
        /// Package identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        package_id: Id,
        /// Remaining after reservation used by staff to prepare training assignment, package, progress, or parent-summary review.
        remaining_after_reservation: SessionBalance,
    },
    /// Staff can see the no remaining sessions training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    NoRemainingSessions {
        /// Package identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        package_id: Id,
        /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
        gate: policy::ReviewGate,
    },
    /// Staff can see the reconciliation required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    ReconciliationRequired {
        /// Package identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
        package_id: Id,
        /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Default)]
/// Training usage policy that reserves the next session or escalates package balance issues.
pub struct UsagePolicy;

impl UsagePolicy {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Caller-reported estimate difference for package/session reconciliation review; never realized savings.
pub struct EstimatedLaborMinutes(u16);

impl EstimatedLaborMinutes {
    /// Accepts a positive labor-minute estimate for training opportunity ranking and outcome comparison.
    pub const fn try_new(value: u16) -> std::result::Result<Self, LaborMinutesError> {
        if value == 0 {
            return Err(LaborMinutesError::Zero);
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for EstimatedLaborMinutes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u16::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Caller-reported minute label retained with a training package/session outcome.
///
/// This serializable value does not prove resolution, staff time, labor effect, or value.
pub struct ReportedLaborMinutes(u16);

impl ReportedLaborMinutes {
    /// Accepts a positive caller-reported minute count for nonclaimable outcome history.
    pub const fn try_new(value: u16) -> std::result::Result<Self, LaborMinutesError> {
        if value == 0 {
            return Err(LaborMinutesError::Zero);
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for ReportedLaborMinutes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u16::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Labor-minute validation failures for training package/session opportunity outcomes.
pub enum LaborMinutesError {
    #[error("training opportunity labor minutes must be positive")]
    /// Rejects zero-minute estimates so labor-reduction reports stay measurable.
    Zero,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
/// Source-backed training package/session opportunity for staff reconciliation or re-enrollment review.
pub struct Opportunity {
    package_id: Id,
    source_record_refs: Vec<crate::source::RecordRef>,
    usage_decision: UsageDecision,
    estimated_minutes: EstimatedLaborMinutes,
}

impl std::fmt::Debug for Opportunity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("training::package::Opportunity([REDACTED])")
    }
}

impl Opportunity {
    /// Builds a reviewable opportunity from package ledger evidence and the usage decision without mutating package balances.
    pub fn from_usage_decision(
        package_id: Id,
        source_record_refs: Vec<crate::source::RecordRef>,
        usage_decision: UsageDecision,
        estimated_minutes: EstimatedLaborMinutes,
    ) -> Result<Self> {
        if source_record_refs.is_empty() {
            return Err(Error::OpportunitySourceEvidenceRequired);
        }
        Ok(Self {
            package_id,
            source_record_refs,
            usage_decision,
            estimated_minutes,
        })
    }

    /// Returns the package whose ledger evidence created this opportunity.
    pub fn package_id(&self) -> &Id {
        &self.package_id
    }

    /// Returns source records staff can inspect before reconciling a package/session opportunity.
    pub fn source_record_refs(&self) -> &[crate::source::RecordRef] {
        &self.source_record_refs
    }

    /// Reports whether the opportunity carries source evidence instead of model-only rationale.
    pub fn has_source_evidence(&self) -> bool {
        !self.source_record_refs.is_empty()
    }
}

impl<'de> Deserialize<'de> for Opportunity {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawOpportunity {
            package_id: Id,
            source_record_refs: Vec<crate::source::RecordRef>,
            usage_decision: UsageDecision,
            estimated_minutes: EstimatedLaborMinutes,
        }

        let raw = RawOpportunity::deserialize(deserializer)?;
        Self::from_usage_decision(
            raw.package_id,
            raw.source_record_refs,
            raw.usage_decision,
            raw.estimated_minutes,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Caller-reported training package/session disposition label retained as nonclaimable evidence; it proves no review, queue, confirmation, suppression, correction, deferral, action, completion, measurement, or value.
pub enum Disposition {
    /// Caller reports a reconciliation-queued label without proving any queue or system-of-record action.
    ReconciliationQueuedLabel,
    /// Caller reports a no-remaining-sessions label without proving confirmation or suppression.
    NoRemainingSessionsLabel,
    /// Caller reports a wrong-source label without adjudicating source truth or authorizing correction.
    WrongSourceLabel,
    /// Caller reports a deferred label without proving staff review or deferral.
    DeferredLabel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Caller-reported training opportunity outcome retained as nonclaimable history.
///
/// Its disposition, source references, and minute labels do not prove review, resolution,
/// measured labor reduction, or value.
pub struct OutcomeRecord {
    package_id: Id,
    source_record_refs: Vec<crate::source::RecordRef>,
    disposition: Disposition,
    before_minutes: EstimatedLaborMinutes,
    reported_minutes: ReportedLaborMinutes,
}

impl OutcomeRecord {
    /// Returns the package label correlated with this caller-reported outcome evidence.
    pub fn package_id(&self) -> &Id {
        &self.package_id
    }

    /// Returns caller-reported source-reference labels without authenticating a reviewer, review, measurement, or labor impact.
    pub fn source_record_refs(&self) -> &[crate::source::RecordRef] {
        &self.source_record_refs
    }

    /// Returns the caller-reported disposition label for evidence reporting.
    pub const fn disposition(&self) -> Disposition {
        self.disposition
    }

    /// Returns the caller-reported baseline-minute estimate.
    pub const fn before_minutes(&self) -> EstimatedLaborMinutes {
        self.before_minutes
    }

    /// Returns the caller-reported workflow-minute label; it is not an actual measurement.
    pub const fn reported_minutes(&self) -> ReportedLaborMinutes {
        self.reported_minutes
    }
}
