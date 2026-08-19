#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Training-domain validation failures that prevent unsupported reports, outcomes, or package usage from entering workflow state.
pub enum Error {
    #[error("training progress report requires evidence before it can be reviewed")]
    /// Staff can see the progress evidence required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    ProgressEvidenceRequired,
    #[error("training outcome claim requires evidence for achieved/readiness claims")]
    /// Staff can see the outcome evidence required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    OutcomeEvidenceRequired,
    #[error("training outcome documentation requires at least one claim")]
    /// Staff can see the outcome claim required training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    OutcomeClaimRequired,
    #[error("training package policy does not define a reusable session balance")]
    /// Staff can see the package has no reusable balance training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    PackageHasNoReusableBalance,
    #[error("training opportunity requires at least one source record")]
    /// Staff can see missing source evidence before a training package/session opportunity enters review.
    OpportunitySourceEvidenceRequired,
}

/// Result type returned by fallible training operations.
pub type Result<T> = std::result::Result<T, Error>;
