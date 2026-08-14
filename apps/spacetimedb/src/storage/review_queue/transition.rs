//! Legal source-state transitions for review-queue rows.

use super::{
    FeedbackOutcomeColumn, ManagerOutcomeColumn, RecommendationColumn, ReviewGateColumn,
    ReviewQueueItemRow, ReviewQueueStatusColumn, StaffDispositionColumn,
};

/// Queue mutation whose source state is checked by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Represents the `Claim` semantic case.
    Claim,
    /// Represents the `AttachRecommendation` semantic case.
    AttachRecommendation,
    /// Represents the `RecordStaffDisposition` semantic case.
    RecordStaffDisposition,
    /// Represents the `RecordManagerOutcome` semantic case.
    RecordManagerOutcome,
    /// Represents the `CaptureOutcome` semantic case.
    CaptureOutcome,
    /// Represents the `BlockUnsafeSideEffect` semantic case.
    BlockUnsafeSideEffect,
}

/// Typed queue transition failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// Represents the `InvalidSourceState` semantic case.
    InvalidSourceState {
        /// Stores the operation component of this boundary value.
        operation: Operation,
        /// Stores the actual component of this boundary value.
        actual: ReviewQueueStatusColumn,
    },
    /// Represents the `ActorAlreadyClaimed` semantic case.
    ActorAlreadyClaimed {
        /// Stores the claimed by component of this boundary value.
        claimed_by: String,
    },
    /// Represents the `ActorDoesNotOwnClaim` semantic case.
    ActorDoesNotOwnClaim,
    /// Represents the `ManagerGateRequired` semantic case.
    ManagerGateRequired,
    /// Represents the `ManagerGateNotRequired` semantic case.
    ManagerGateNotRequired,
    /// Represents the `OutcomeDoesNotMatchDisposition` semantic case.
    OutcomeDoesNotMatchDisposition,
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidSourceState { operation, actual } => write!(
                formatter,
                "{operation} is not legal from {}",
                super::codec::status_label(*actual)
            ),
            Self::ActorAlreadyClaimed { claimed_by } => {
                write!(formatter, "review item is already claimed by {claimed_by}")
            }
            Self::ActorDoesNotOwnClaim => {
                formatter.write_str("actor does not own the review item claim")
            }
            Self::ManagerGateRequired => formatter.write_str("manager approval gate is required"),
            Self::ManagerGateNotRequired => {
                formatter.write_str("manager approval gate is not required")
            }
            Self::OutcomeDoesNotMatchDisposition => {
                formatter.write_str("captured outcome does not match the recorded disposition")
            }
        }
    }
}

impl core::fmt::Display for Operation {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Claim => "claim",
            Self::AttachRecommendation => "attach recommendation",
            Self::RecordStaffDisposition => "record staff disposition",
            Self::RecordManagerOutcome => "record manager outcome",
            Self::CaptureOutcome => "capture outcome",
            Self::BlockUnsafeSideEffect => "block unsafe side effect",
        })
    }
}

/// Canonical result used by this module.
pub type Result<T> = core::result::Result<T, Error>;

fn require(
    row: &ReviewQueueItemRow,
    operation: Operation,
    expected: ReviewQueueStatusColumn,
) -> Result<()> {
    if row.status == expected {
        Ok(())
    } else {
        Err(Error::InvalidSourceState {
            operation,
            actual: row.status,
        })
    }
}

/// Returns the aggregate claim.
pub fn claim(row: &mut ReviewQueueItemRow, actor_id: String) -> Result<()> {
    require(
        row,
        Operation::Claim,
        ReviewQueueStatusColumn::PendingStaffReview,
    )?;
    if let Some(claimed_by) = &row.claimed_by_actor_id {
        return Err(Error::ActorAlreadyClaimed {
            claimed_by: claimed_by.clone(),
        });
    }
    row.claimed_by_actor_id = Some(actor_id);
    row.status = ReviewQueueStatusColumn::ClaimedByStaff;
    Ok(())
}

/// Attaches recommendation after checking ownership and source state.
pub fn attach_recommendation(
    row: &mut ReviewQueueItemRow,
    actor_id: &str,
    recommendation: RecommendationColumn,
) -> Result<()> {
    require(
        row,
        Operation::AttachRecommendation,
        ReviewQueueStatusColumn::ClaimedByStaff,
    )?;
    require_claim_owner(row, actor_id)?;
    row.recommendation = Some(recommendation);
    Ok(())
}

/// Records staff disposition after checking the legal source state.
pub fn record_staff_disposition(
    row: &mut ReviewQueueItemRow,
    actor_id: &str,
    disposition: StaffDispositionColumn,
) -> Result<()> {
    require(
        row,
        Operation::RecordStaffDisposition,
        ReviewQueueStatusColumn::ClaimedByStaff,
    )?;
    require_claim_owner(row, actor_id)?;
    let manager_gate = row
        .required_review_gates
        .contains(&ReviewGateColumn::ManagerApproval);
    let target_status = match disposition {
        StaffDispositionColumn::RecommendForManagerApproval if manager_gate => {
            ReviewQueueStatusColumn::PendingManagerApproval
        }
        StaffDispositionColumn::RecommendForManagerApproval => {
            return Err(Error::ManagerGateNotRequired);
        }
        StaffDispositionColumn::CompleteWithoutManagerApproval if !manager_gate => {
            ReviewQueueStatusColumn::ReadyForOutcome
        }
        StaffDispositionColumn::CompleteWithoutManagerApproval => {
            return Err(Error::ManagerGateRequired);
        }
        StaffDispositionColumn::Defer => ReviewQueueStatusColumn::ReadyForOutcome,
    };
    row.staff_disposition = Some(disposition);
    row.status = target_status;
    Ok(())
}

/// Records manager outcome after checking the legal source state.
pub fn record_manager_outcome(
    row: &mut ReviewQueueItemRow,
    outcome: ManagerOutcomeColumn,
) -> Result<()> {
    require(
        row,
        Operation::RecordManagerOutcome,
        ReviewQueueStatusColumn::PendingManagerApproval,
    )?;
    if !row
        .required_review_gates
        .contains(&ReviewGateColumn::ManagerApproval)
    {
        return Err(Error::ManagerGateRequired);
    }
    row.manager_outcome = Some(outcome);
    row.status = ReviewQueueStatusColumn::ReadyForOutcome;
    Ok(())
}

/// Captures outcome after validating the recorded disposition.
pub fn capture_outcome(row: &mut ReviewQueueItemRow, outcome: FeedbackOutcomeColumn) -> Result<()> {
    require(
        row,
        Operation::CaptureOutcome,
        ReviewQueueStatusColumn::ReadyForOutcome,
    )?;
    let matches_disposition = match (row.manager_outcome, row.staff_disposition) {
        (Some(ManagerOutcomeColumn::Rejected), _) => {
            outcome == FeedbackOutcomeColumn::SuppressedByManager
        }
        (Some(ManagerOutcomeColumn::Deferred), _) => outcome == FeedbackOutcomeColumn::Deferred,
        (Some(ManagerOutcomeColumn::Approved), _) => !matches!(
            outcome,
            FeedbackOutcomeColumn::Deferred | FeedbackOutcomeColumn::SuppressedByManager
        ),
        (None, Some(StaffDispositionColumn::Defer)) => outcome == FeedbackOutcomeColumn::Deferred,
        (None, Some(StaffDispositionColumn::CompleteWithoutManagerApproval)) => !matches!(
            outcome,
            FeedbackOutcomeColumn::Deferred | FeedbackOutcomeColumn::SuppressedByManager
        ),
        _ => false,
    };
    if !matches_disposition {
        return Err(Error::OutcomeDoesNotMatchDisposition);
    }
    row.status = ReviewQueueStatusColumn::OutcomeRecorded;
    Ok(())
}

fn require_claim_owner(row: &ReviewQueueItemRow, actor_id: &str) -> Result<()> {
    if row.claimed_by_actor_id.as_deref() == Some(actor_id) {
        Ok(())
    } else {
        Err(Error::ActorDoesNotOwnClaim)
    }
}

/// Marks the queue item blocked for unsafe side effect.
pub fn block_unsafe_side_effect(row: &mut ReviewQueueItemRow) -> Result<()> {
    if row.status == ReviewQueueStatusColumn::OutcomeRecorded {
        return Err(Error::InvalidSourceState {
            operation: Operation::BlockUnsafeSideEffect,
            actual: row.status,
        });
    }
    row.status = ReviewQueueStatusColumn::Blocked;
    Ok(())
}
