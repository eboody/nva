use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Outcome-claim status used in trainer evidence and parent-facing documentation review.
pub enum ClaimStatus {
    /// Staff can see the achieved training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    Achieved,
    /// Staff can see the readiness training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    Readiness,
    /// Staff can see the deferred training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    Deferred,
    /// Staff can see the not assessed training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    NotAssessed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Evidence bundle used to promote an outcome claim into reviewed documentation.
pub struct ClaimEvidence {
    /// Outcome used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub outcome: Outcome,
    /// Status used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub status: ClaimStatus,
    /// Evidence used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub evidence: Vec<EvidenceId>,
    /// Milestones used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub milestones: Vec<curriculum::milestone::Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Outcome claim whose achieved/readiness status cannot exist without supporting evidence.
pub struct Claim {
    /// Outcome used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub outcome: Outcome,
    /// Status used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub status: ClaimStatus,
    evidence: Vec<EvidenceId>,
    milestones: Vec<curriculum::milestone::Id>,
}

impl Claim {
    /// Builds this training value from evidence data.
    pub fn from_evidence(value: ClaimEvidence) -> Result<Self> {
        if matches!(value.status, ClaimStatus::Achieved | ClaimStatus::Readiness)
            && value.evidence.is_empty()
        {
            return Err(Error::OutcomeEvidenceRequired);
        }
        Ok(Self {
            outcome: value.outcome,
            status: value.status,
            evidence: value.evidence,
            milestones: value.milestones,
        })
    }
    /// Returns the trainer/source evidence that supports this outcome claim.
    pub fn evidence(&self) -> &[EvidenceId] {
        &self.evidence
    }
    /// Returns the milestones value used by training assignment, progress, package, or parent-summary review.
    pub fn milestones(&self) -> &[curriculum::milestone::Id] {
        &self.milestones
    }
}

impl<'de> Deserialize<'de> for Claim {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::from_evidence(ClaimEvidence::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Non-empty outcome-claim list for documentation packets.
///
/// Requiring at least one claim at construction time prevents empty graduation/readiness
/// documentation from reaching member-facing approval queues, so reviewers check concrete
/// trainer evidence instead of spending labor on unsupported shells.
pub struct Claims(NonEmpty<Claim>);

impl Claims {
    /// Promotes outcome claims into a non-empty packet before documentation review can begin.
    pub fn try_new(claims: Vec<Claim>) -> Result<Self> {
        NonEmpty::from_vec(claims)
            .map(Self)
            .ok_or(Error::OutcomeClaimRequired)
    }

    /// Returns the first outcome claim; total because documentation cannot exist without at least one claim.
    pub fn first(&self) -> &Claim {
        self.0.first()
    }

    /// Iterates over every evidence-backed claim in this documentation packet.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Claim> {
        self.0.iter()
    }

    /// Returns the evidence-backed claims in this documentation packet.
    pub fn as_vec_refs(&self) -> Vec<&Claim> {
        self.iter().collect()
    }
}

impl<'de> Deserialize<'de> for Claims {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(Vec::<Claim>::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Training outcome documentation packet for customer/account history and manager review.
pub struct Documentation {
    /// Documentation identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub documentation_id: OutcomeDocumentationId,
    /// Enrollment identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub enrollment_id: enrollment::Id,
    /// Pet receiving the training service or parent-facing progress update.
    pub pet_id: PetId,
    /// Location identifier used by staff to prepare training assignment, package, progress, or parent-summary review.
    pub location_id: LocationId,
    claims: Claims,
    review: OutcomeReviewState,
}

impl Documentation {
    /// Returns the first outcome claim; total because documentation cannot exist without at least one claim.
    pub fn first_claim(&self) -> &Claim {
        self.claims.first()
    }
    /// Returns the review value used by training assignment, progress, package, or parent-summary review.
    pub fn review(&self) -> &OutcomeReviewState {
        &self.review
    }
}
