use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Non-empty trainer/source evidence list for progress reports.
///
/// Requiring evidence at construction time prevents draft automation from creating
/// unsupported parent-facing summaries and reduces labor-review cost: reviewers inspect
/// cited trainer/source facts instead of hunting for absent evidence after the report exists.
pub struct EvidenceSet(NonEmpty<ProgressEvidence>);

impl EvidenceSet {
    /// Promotes report evidence into a non-empty set before report review can begin.
    pub fn try_new(evidence: Vec<ProgressEvidence>) -> Result<Self> {
        NonEmpty::from_vec(evidence)
            .map(Self)
            .ok_or(Error::ProgressEvidenceRequired)
    }

    /// Returns the first trainer/source evidence item; total because the set is non-empty by construction.
    pub fn first(&self) -> &ProgressEvidence {
        self.0.first()
    }

    /// Iterates over all trainer/source evidence that supports this report.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ProgressEvidence> {
        self.0.iter()
    }

    /// Returns the trainer/source evidence that supports this report.
    pub fn as_vec_refs(&self) -> Vec<&ProgressEvidence> {
        self.iter().collect()
    }
}

impl<'de> Deserialize<'de> for EvidenceSet {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(Vec::<ProgressEvidence>::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Training progress report carrying session evidence, milestones, and approval state.
pub struct Report {
    /// Report identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub report_id: ProgressReportId,
    /// Enrollment identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub enrollment_id: enrollment::Id,
    /// Session ref used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub session_ref: SessionRef,
    evidence: EvidenceSet,
    milestones: Vec<curriculum::Progress>,
    approval: ApprovalState,
}

impl Report {
    /// Returns the first trainer/source evidence item; total because reports cannot exist without evidence.
    pub fn first_evidence(&self) -> &ProgressEvidence {
        self.evidence.first()
    }
    /// Returns the milestones value used by training assignment, progress, package, or parent-summary review.
    pub fn milestones(&self) -> &[curriculum::Progress] {
        &self.milestones
    }
    /// Returns the approval value used by training assignment, progress, package, or parent-summary review.
    pub fn approval(&self) -> &ApprovalState {
        &self.approval
    }
}
