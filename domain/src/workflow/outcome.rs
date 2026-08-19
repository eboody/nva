use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Workflow result carrying a summary plus one coherent, evidence-bearing outcome variant.
pub struct Result<T> {
    /// Workflow summary value preserved for staff review and audit evidence.
    summary: Summary,
    /// Evidence-bearing workflow outcome. The variant owns the fields legal for that state.
    outcome: Outcome<T>,
}

impl<T> Result<T> {
    /// Records a workflow outcome that stopped at a human review gate.
    pub fn needs_human_review(
        summary: Summary,
        human_review_reason: ReviewReason,
        recommended_actions: Vec<RecommendedAction>,
        risk_flags: Vec<RiskFlag>,
        verification: Vec<VerificationNote>,
    ) -> std::result::Result<Self, Error> {
        ensure_verification_evidence(&verification)?;
        Ok(Self {
            summary,
            outcome: Outcome::NeedsHumanReview {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            },
        })
    }

    /// Records a workflow outcome rejected by deterministic policy evidence.
    pub fn rejected_by_policy(
        summary: Summary,
        human_review_reason: ReviewReason,
        recommended_actions: Vec<RecommendedAction>,
        risk_flags: Vec<RiskFlag>,
        verification: Vec<VerificationNote>,
    ) -> std::result::Result<Self, Error> {
        ensure_verification_evidence(&verification)?;
        Ok(Self {
            summary,
            outcome: Outcome::RejectedByPolicy {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            },
        })
    }

    /// Records a workflow outcome that needs more source/staff information.
    pub fn needs_more_information(
        summary: Summary,
        human_review_reason: ReviewReason,
        recommended_actions: Vec<RecommendedAction>,
        risk_flags: Vec<RiskFlag>,
        verification: Vec<VerificationNote>,
    ) -> std::result::Result<Self, Error> {
        ensure_verification_evidence(&verification)?;
        Ok(Self {
            summary,
            outcome: Outcome::NeedsMoreInformation {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            },
        })
    }

    /// Records a workflow outcome that failed safely without producing live side effects.
    pub fn failed_safely(
        summary: Summary,
        human_review_reason: ReviewReason,
        recommended_actions: Vec<RecommendedAction>,
        risk_flags: Vec<RiskFlag>,
        verification: Vec<VerificationNote>,
    ) -> std::result::Result<Self, Error> {
        ensure_verification_evidence(&verification)?;
        Ok(Self {
            summary,
            outcome: Outcome::FailedSafely {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            },
        })
    }

    /// Summary preserved for staff review and audit evidence.
    pub const fn summary(&self) -> &Summary {
        &self.summary
    }

    /// Coherent evidence-bearing outcome variant.
    pub const fn outcome(&self) -> &Outcome<T> {
        &self.outcome
    }

    /// Stable status code used only by explicit versioned DTO/storage projections.
    pub const fn status(&self) -> Status {
        self.outcome.status()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Evidence-bearing result variant. Each outcome owns exactly the fields legal for its status.
pub enum Outcome<T> {
    /// Workflow completed with structured output and verification evidence.
    Completed {
        /// Typed output accepted by the owning workflow validator.
        structured_output: T,
        /// Staff-visible next actions created from the accepted output.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving why the output can be treated as completed workflow output.
        verification: Vec<VerificationNote>,
    },
    /// Workflow stopped at a human review gate with an explicit reason.
    NeedsHumanReview {
        /// Review-gate evidence explaining why automation must stop.
        human_review_reason: ReviewReason,
        /// Staff-visible next actions created before the review gate.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving why review is required.
        verification: Vec<VerificationNote>,
    },
    /// Workflow was rejected by policy with reviewable evidence.
    RejectedByPolicy {
        /// Policy/review evidence explaining the rejection.
        human_review_reason: ReviewReason,
        /// Staff-visible next actions created before rejection.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving why the policy rejection is valid.
        verification: Vec<VerificationNote>,
    },
    /// Workflow needs more source/staff information before continuing.
    NeedsMoreInformation {
        /// Evidence explaining the missing information boundary.
        human_review_reason: ReviewReason,
        /// Staff-visible next actions for gathering missing proof.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving which facts were insufficient.
        verification: Vec<VerificationNote>,
    },
    /// Workflow failed safely without live side effects.
    FailedSafely {
        /// Evidence explaining the safe-failure boundary.
        human_review_reason: ReviewReason,
        /// Staff-visible next actions after safe failure.
        recommended_actions: Vec<RecommendedAction>,
        /// Risk markers retained for reports and review.
        risk_flags: Vec<RiskFlag>,
        /// Evidence proving no unsafe completion was manufactured.
        verification: Vec<VerificationNote>,
    },
}

impl<T> Outcome<T> {
    /// Stable status code used only by explicit versioned DTO/storage projections.
    pub const fn status(&self) -> Status {
        match self {
            Self::Completed { .. } => Status::Completed,
            Self::NeedsHumanReview { .. } => Status::NeedsHumanReview,
            Self::RejectedByPolicy { .. } => Status::RejectedByPolicy,
            Self::NeedsMoreInformation { .. } => Status::NeedsMoreInformation,
            Self::FailedSafely { .. } => Status::FailedSafely,
        }
    }

    /// Recommended staff/automation actions for this outcome.
    pub fn recommended_actions(&self) -> &[RecommendedAction] {
        match self {
            Self::Completed {
                recommended_actions,
                ..
            }
            | Self::NeedsHumanReview {
                recommended_actions,
                ..
            }
            | Self::RejectedByPolicy {
                recommended_actions,
                ..
            }
            | Self::NeedsMoreInformation {
                recommended_actions,
                ..
            }
            | Self::FailedSafely {
                recommended_actions,
                ..
            } => recommended_actions,
        }
    }

    /// Risk flags carried as review and reporting evidence for this outcome.
    pub fn risk_flags(&self) -> &[RiskFlag] {
        match self {
            Self::Completed { risk_flags, .. }
            | Self::NeedsHumanReview { risk_flags, .. }
            | Self::RejectedByPolicy { risk_flags, .. }
            | Self::NeedsMoreInformation { risk_flags, .. }
            | Self::FailedSafely { risk_flags, .. } => risk_flags,
        }
    }

    /// Verification evidence carried by the outcome.
    pub fn verification(&self) -> &[VerificationNote] {
        match self {
            Self::Completed { verification, .. }
            | Self::NeedsHumanReview { verification, .. }
            | Self::RejectedByPolicy { verification, .. }
            | Self::NeedsMoreInformation { verification, .. }
            | Self::FailedSafely { verification, .. } => verification,
        }
    }

    /// Human review evidence for variants that legally stop at a review gate.
    pub fn human_review_reason(&self) -> Option<&ReviewReason> {
        match self {
            Self::Completed { .. } => None,
            Self::NeedsHumanReview {
                human_review_reason,
                ..
            }
            | Self::RejectedByPolicy {
                human_review_reason,
                ..
            }
            | Self::NeedsMoreInformation {
                human_review_reason,
                ..
            }
            | Self::FailedSafely {
                human_review_reason,
                ..
            } => Some(human_review_reason),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by workflow result/outcome construction and rehydration.
pub enum Error {
    /// Serialized caller output attempted to claim workflow completion.
    #[error("serialized completed workflow outcome is not accepted")]
    SerializedCompletedOutcomeNotAccepted,
    /// Completed outcome carried a human review reason,
    /// A stopped or rejected outcome carried completed structured output evidence.
    #[error("non-completed workflow outcome must not carry structured output")]
    NonCompletedOutcomeMustNotCarryStructuredOutput,
    /// Human-review outcome omitted the review reason evidence.
    #[error("workflow outcome needing human review requires review reason evidence")]
    HumanReviewOutcomeRequiresReviewReason,
    /// Policy-rejected outcome omitted the review/policy reason evidence.
    #[error("policy-rejected workflow outcome requires review reason evidence")]
    PolicyRejectedOutcomeRequiresReviewReason,
    /// More-information outcome omitted the missing-information reason evidence.
    #[error("workflow outcome needing more information requires review reason evidence")]
    NeedsMoreInformationOutcomeRequiresReviewReason,
    /// Safe-failure outcome omitted the reason/evidence that explains the stop.
    #[error("failed-safe workflow outcome requires review reason evidence")]
    FailedSafelyOutcomeRequiresReviewReason,
    /// Outcome variants must carry at least one verification note before they can enter reports.
    #[error("workflow outcome requires verification evidence")]
    VerificationEvidenceRequired,
}

fn ensure_verification_evidence(
    verification: &[VerificationNote],
) -> std::result::Result<(), Error> {
    if verification.is_empty() {
        return Err(Error::VerificationEvidenceRequired);
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
/// Canonical workflow-result wire projection.
struct WireResult<T> {
    status: Status,
    summary: Summary,
    structured_output: Option<T>,
    recommended_actions: Vec<RecommendedAction>,
    risk_flags: Vec<RiskFlag>,
    verification: Vec<VerificationNote>,
    human_review_reason: Option<ReviewReason>,
}

impl<T> TryFrom<WireResult<T>> for Result<T> {
    type Error = Error;

    fn try_from(value: WireResult<T>) -> std::result::Result<Self, Self::Error> {
        if value.status != Status::Completed && value.structured_output.is_some() {
            return Err(Error::NonCompletedOutcomeMustNotCarryStructuredOutput);
        }
        match value.status {
            Status::Completed => Err(Error::SerializedCompletedOutcomeNotAccepted),
            Status::NeedsHumanReview => {
                let Some(reason) = value.human_review_reason else {
                    return Err(Error::HumanReviewOutcomeRequiresReviewReason);
                };
                Self::needs_human_review(
                    value.summary,
                    reason,
                    value.recommended_actions,
                    value.risk_flags,
                    value.verification,
                )
            }
            Status::RejectedByPolicy => {
                let Some(reason) = value.human_review_reason else {
                    return Err(Error::PolicyRejectedOutcomeRequiresReviewReason);
                };
                Self::rejected_by_policy(
                    value.summary,
                    reason,
                    value.recommended_actions,
                    value.risk_flags,
                    value.verification,
                )
            }
            Status::NeedsMoreInformation => {
                let Some(reason) = value.human_review_reason else {
                    return Err(Error::NeedsMoreInformationOutcomeRequiresReviewReason);
                };
                Self::needs_more_information(
                    value.summary,
                    reason,
                    value.recommended_actions,
                    value.risk_flags,
                    value.verification,
                )
            }
            Status::FailedSafely => {
                let Some(reason) = value.human_review_reason else {
                    return Err(Error::FailedSafelyOutcomeRequiresReviewReason);
                };
                Self::failed_safely(
                    value.summary,
                    reason,
                    value.recommended_actions,
                    value.risk_flags,
                    value.verification,
                )
            }
        }
    }
}

impl<T> From<Result<T>> for WireResult<T> {
    fn from(value: Result<T>) -> Self {
        let status = value.status();
        match value.outcome {
            Outcome::Completed {
                structured_output,
                recommended_actions,
                risk_flags,
                verification,
            } => Self {
                status,
                summary: value.summary,
                structured_output: Some(structured_output),
                recommended_actions,
                risk_flags,
                verification,
                human_review_reason: None,
            },
            Outcome::NeedsHumanReview {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::RejectedByPolicy {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::NeedsMoreInformation {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::FailedSafely {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            } => Self {
                status,
                summary: value.summary,
                structured_output: None,
                recommended_actions,
                risk_flags,
                verification,
                human_review_reason: Some(human_review_reason),
            },
        }
    }
}

impl<T> Serialize for Result<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status = self.status();
        match &self.outcome {
            Outcome::Completed {
                structured_output,
                recommended_actions,
                risk_flags,
                verification,
            } => WireResult {
                status,
                summary: self.summary.clone(),
                structured_output: Some(structured_output),
                recommended_actions: recommended_actions.clone(),
                risk_flags: risk_flags.clone(),
                verification: verification.clone(),
                human_review_reason: None,
            }
            .serialize(serializer),
            Outcome::NeedsHumanReview {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::RejectedByPolicy {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::NeedsMoreInformation {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            }
            | Outcome::FailedSafely {
                human_review_reason,
                recommended_actions,
                risk_flags,
                verification,
            } => WireResult::<&T> {
                status,
                summary: self.summary.clone(),
                structured_output: None,
                recommended_actions: recommended_actions.clone(),
                risk_flags: risk_flags.clone(),
                verification: verification.clone(),
                human_review_reason: Some(human_review_reason.clone()),
            }
            .serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for Result<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let projection = WireResult::deserialize(deserializer)?;
        Self::try_from(projection).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Normalized lifecycle states used to reconcile source-system data with domain workflows.
pub enum Status {
    /// Completed workflow state, command, or review outcome.
    Completed,
    /// Needs human review workflow state, command, or review outcome.
    NeedsHumanReview,
    /// Rejected by policy workflow state, command, or review outcome.
    RejectedByPolicy,
    /// Needs more information workflow state, command, or review outcome.
    NeedsMoreInformation,
    /// Failed safely workflow state, command, or review outcome.
    FailedSafely,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Recommended next action for staff, managers, or automation after evaluating a workflow.
pub enum RecommendedAction {
    /// Internal task workflow state, command, or review outcome.
    InternalTask {
        /// Workflow title value preserved for staff review and audit evidence.
        title: task::Title,
        /// Workflow body value preserved for staff review and audit evidence.
        body: task::Body,
    },
    /// Draft message workflow state, command, or review outcome.
    DraftMessage {
        /// Workflow channel value preserved for staff review and audit evidence.
        channel: message::Channel,
        /// Workflow body value preserved for staff review and audit evidence.
        body: message::Body,
    },
    /// Update status workflow state, command, or review outcome.
    UpdateStatus {
        /// Workflow target value preserved for staff review and audit evidence.
        target: status_update::Target,
    },
    /// Request human review workflow state, command, or review outcome.
    RequestHumanReview(policy::ReviewGate),
}
