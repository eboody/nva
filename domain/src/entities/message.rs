//! Message aggregates and target-bound queue authority.

use serde::{Deserialize, Deserializer, Serialize};

use super::{
    actor::ActorRef,
    approval,
    identifiers::{CustomerId, IncidentId, MessageId, PetId},
    reservation,
};
use crate::{message, policy};

/// Message aggregate vocabulary and checked lifecycle evidence.
pub mod message_record {
    use chrono::{DateTime, Utc};
    use serde::Serialize;

    use super::{ActorRef, MessageId, approval, message, policy};

    /// Message aggregate construction and lifecycle-promotion failures.
    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum Error {
        #[error("outbound draft cannot carry queued, attempted, or delivered status")]
        /// Represents the `DraftCannotCarryDeliveryStatus` semantic case.
        DraftCannotCarryDeliveryStatus,
        #[error("queued or approved outbound message requires approval decision evidence")]
        /// Represents the `QueuedOrApprovedOutboundRequiresApprovalEvidence` semantic case.
        QueuedOrApprovedOutboundRequiresApprovalEvidence,
        #[error("inbound messages cannot carry outbound delivery lifecycle status")]
        /// Represents the `InboundCannotCarryOutboundDeliveryStatus` semantic case.
        InboundCannotCarryOutboundDeliveryStatus,
        #[error("outbound sent message requires attempted, delivered, or failed status")]
        /// Represents the `SentRequiresAttemptedDeliveredOrFailedStatus` semantic case.
        SentRequiresAttemptedDeliveredOrFailedStatus,
        #[error(
            "message approval evidence must be an approved decision for the requested message and gate"
        )]
        /// Represents the `ApprovalEvidenceMismatch` semantic case.
        ApprovalEvidenceMismatch,
        #[error("queued or sent message rehydration requires opaque queue authorization")]
        /// Generic serde cannot promote historical approval fields into executable lifecycle state.
        ExecutableRehydrationRequiresQueueAuthorization,
    }

    /// Result alias for message aggregate construction and lifecycle promotion.
    pub type Result<T> = std::result::Result<T, Error>;

    /// Serializable historical approval evidence for a message lifecycle.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct ApprovalEvidence {
        /// Approval decision id when the evidence came from an owned approval record.
        approval_id: Option<approval::Id>,
        /// Exact message target approved by the owned decision.
        message_id: MessageId,
        /// Review gate that approved or historically guarded this message lifecycle.
        gate: policy::ReviewGate,
        /// Actor that approved queueing when known from the approval record.
        decided_by: Option<ActorRef>,
        /// Decision time when known from the approval record.
        decided_at: Option<DateTime<Utc>>,
    }

    impl ApprovalEvidence {
        pub(super) const fn is_owned_decision(&self) -> bool {
            self.approval_id.is_some() && self.decided_by.is_some() && self.decided_at.is_some()
        }

        /// Approval record id retained as historical evidence, when available.
        pub const fn approval_id(&self) -> Option<approval::Id> {
            self.approval_id
        }

        /// Exact message target retained from the approval decision.
        pub const fn message_id(&self) -> MessageId {
            self.message_id
        }

        /// Review gate retained as historical evidence.
        pub fn gate(&self) -> &policy::ReviewGate {
            &self.gate
        }
    }

    /// Checked message lifecycle; variants carry exactly the evidence legal for their phase.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Lifecycle {
        /// Inbound source message with no outbound approval or queue authority.
        InboundReceived,
        /// Outbound draft or review-requested draft that has not entered the queue.
        OutboundDraft {
            /// Draft-side status, limited to statuses that cannot imply send authority.
            status: message::Status,
            /// Review gate requested for this draft, if any.
            approval_gate: Option<policy::ReviewGate>,
        },
        /// Outbound message admitted to the queue by approval evidence and queue capability.
        OutboundQueued {
            /// Queue-side status before provider delivery evidence exists.
            status: message::Status,
            /// Historical approval evidence that justified queue admission.
            approval_evidence: ApprovalEvidence,
        },
        /// Outbound delivery path with approval evidence and attempt/result status.
        OutboundSent {
            /// Attempt/result status.
            status: message::Status,
            /// Historical approval evidence that justified the outbound send path.
            approval_evidence: ApprovalEvidence,
        },
    }

    impl Lifecycle {
        pub(super) fn try_new(
            direction: message::Direction,
            status: message::Status,
            approval_gate: Option<policy::ReviewGate>,
        ) -> Result<Self> {
            match direction {
                message::Direction::InboundReceived => {
                    if matches!(
                        status,
                        message::Status::ApprovedToQueue
                            | message::Status::Queued
                            | message::Status::SendAttempted
                            | message::Status::Delivered
                    ) {
                        return Err(Error::InboundCannotCarryOutboundDeliveryStatus);
                    }
                    Ok(Self::InboundReceived)
                }
                message::Direction::OutboundDraft => {
                    if !matches!(
                        status,
                        message::Status::DraftCreated
                            | message::Status::ApprovalRequested
                            | message::Status::Suppressed
                            | message::Status::Cancelled
                    ) {
                        return Err(Error::DraftCannotCarryDeliveryStatus);
                    }
                    Ok(Self::OutboundDraft {
                        status,
                        approval_gate,
                    })
                }
                message::Direction::OutboundQueued | message::Direction::OutboundSent => {
                    Err(Error::QueuedOrApprovedOutboundRequiresApprovalEvidence)
                }
            }
        }

        pub(super) fn try_from_persisted(
            message_id: MessageId,
            direction: message::Direction,
            status: message::Status,
            approval_gate: Option<policy::ReviewGate>,
            approval_evidence: Option<ApprovalEvidence>,
        ) -> Result<Self> {
            match direction {
                message::Direction::InboundReceived | message::Direction::OutboundDraft => {
                    if approval_evidence.is_some() {
                        return Err(Error::ApprovalEvidenceMismatch);
                    }
                    Self::try_new(direction, status, approval_gate)
                }
                message::Direction::OutboundQueued => {
                    let evidence = approval_evidence
                        .filter(ApprovalEvidence::is_owned_decision)
                        .ok_or(Error::QueuedOrApprovedOutboundRequiresApprovalEvidence)?;
                    if evidence.message_id() != message_id
                        || approval_gate.as_ref() != Some(evidence.gate())
                    {
                        return Err(Error::ApprovalEvidenceMismatch);
                    }
                    if !matches!(
                        status,
                        message::Status::ApprovedToQueue | message::Status::Queued
                    ) {
                        return Err(Error::DraftCannotCarryDeliveryStatus);
                    }
                    Ok(Self::OutboundQueued {
                        status,
                        approval_evidence: evidence,
                    })
                }
                message::Direction::OutboundSent => {
                    let evidence = approval_evidence
                        .filter(ApprovalEvidence::is_owned_decision)
                        .ok_or(Error::QueuedOrApprovedOutboundRequiresApprovalEvidence)?;
                    if evidence.message_id() != message_id
                        || approval_gate.as_ref() != Some(evidence.gate())
                    {
                        return Err(Error::ApprovalEvidenceMismatch);
                    }
                    if !matches!(
                        status,
                        message::Status::SendAttempted
                            | message::Status::Delivered
                            | message::Status::Failed
                    ) {
                        return Err(Error::SentRequiresAttemptedDeliveredOrFailedStatus);
                    }
                    Ok(Self::OutboundSent {
                        status,
                        approval_evidence: evidence,
                    })
                }
            }
        }

        pub(super) fn direction(&self) -> message::Direction {
            match self {
                Self::InboundReceived => message::Direction::InboundReceived,
                Self::OutboundDraft { .. } => message::Direction::OutboundDraft,
                Self::OutboundQueued { .. } => message::Direction::OutboundQueued,
                Self::OutboundSent { .. } => message::Direction::OutboundSent,
            }
        }

        pub(super) fn status(&self) -> message::Status {
            match self {
                Self::InboundReceived => message::Status::DraftCreated,
                Self::OutboundDraft { status, .. }
                | Self::OutboundQueued { status, .. }
                | Self::OutboundSent { status, .. } => *status,
            }
        }

        pub(super) fn approval_gate(&self) -> Option<policy::ReviewGate> {
            match self {
                Self::InboundReceived => None,
                Self::OutboundDraft { approval_gate, .. } => approval_gate.clone(),
                Self::OutboundQueued {
                    approval_evidence, ..
                }
                | Self::OutboundSent {
                    approval_evidence, ..
                } => Some(approval_evidence.gate.clone()),
            }
        }

        pub(super) fn approval_evidence(&self) -> Option<ApprovalEvidence> {
            match self {
                Self::OutboundQueued {
                    approval_evidence, ..
                }
                | Self::OutboundSent {
                    approval_evidence, ..
                } => Some(approval_evidence.clone()),
                Self::InboundReceived | Self::OutboundDraft { .. } => None,
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
/// Customer/internal message record that tracks subject, channel, draft/reference body, approval, and delivery state.
pub struct Message {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    id: MessageId,
    /// Subject retained from source records for staff review, safety gates, and workflow joins.
    subject: MessageSubject,
    /// Channel retained from source records for staff review, safety gates, and workflow joins.
    channel: message::Channel,
    /// Body ref retained from source records for staff review, safety gates, and workflow joins.
    body_ref: message::BodyRef,
    /// Checked lifecycle evidence that replaces independent direction/status/gate products.
    lifecycle: message_record::Lifecycle,
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Deserialize)]
struct RawMessage {
    id: MessageId,
    subject: MessageSubject,
    direction: message::Direction,
    channel: message::Channel,
    status: message::Status,
    body_ref: message::BodyRef,
    approval_gate: Option<policy::ReviewGate>,
    approval_evidence: Option<serde::de::IgnoredAny>,
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Serialize)]
struct SerializedMessage<'a> {
    id: MessageId,
    subject: &'a MessageSubject,
    direction: message::Direction,
    channel: message::Channel,
    status: message::Status,
    body_ref: &'a message::BodyRef,
    approval_gate: Option<policy::ReviewGate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approval_evidence: Option<&'a message_record::ApprovalEvidence>,
    audit_refs: &'a [crate::audit::EventId],
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let approval_evidence = self.lifecycle.approval_evidence();
        SerializedMessage {
            id: self.id,
            subject: &self.subject,
            direction: self.direction(),
            channel: self.channel,
            status: self.status(),
            body_ref: &self.body_ref,
            approval_gate: self.approval_gate(),
            approval_evidence: approval_evidence.as_ref(),
            audit_refs: &self.audit_refs,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawMessage::deserialize(deserializer)?;
        Self::try_from_persisted(raw).map_err(serde::de::Error::custom)
    }
}

impl Message {
    fn try_from_persisted(raw: RawMessage) -> message_record::Result<Self> {
        if raw.approval_evidence.is_some()
            || matches!(
                raw.status,
                message::Status::ApprovedToQueue
                    | message::Status::Queued
                    | message::Status::SendAttempted
                    | message::Status::Delivered
            )
        {
            return Err(message_record::Error::ExecutableRehydrationRequiresQueueAuthorization);
        }
        let lifecycle = message_record::Lifecycle::try_from_persisted(
            raw.id,
            raw.direction,
            raw.status,
            raw.approval_gate,
            None,
        )?;
        Ok(Self {
            id: raw.id,
            subject: raw.subject,
            channel: raw.channel,
            body_ref: raw.body_ref,
            lifecycle,
            audit_refs: raw.audit_refs,
        })
    }

    /// Message identifier used by workflow, storage, and review joins.
    pub fn id(&self) -> MessageId {
        self.id
    }

    /// Subject this message refers to.
    pub fn subject(&self) -> &MessageSubject {
        &self.subject
    }

    /// Direction derived from the checked lifecycle variant.
    pub fn direction(&self) -> message::Direction {
        self.lifecycle.direction()
    }

    /// Delivery/review status derived from the checked lifecycle variant.
    pub fn status(&self) -> message::Status {
        self.lifecycle.status()
    }

    /// Channel selected for this message.
    pub fn channel(&self) -> message::Channel {
        self.channel
    }

    /// Body reference containing draft/source evidence.
    pub fn body_ref(&self) -> &message::BodyRef {
        &self.body_ref
    }

    /// Serializable review gate evidence attached to lifecycle variants that require it.
    pub fn approval_gate(&self) -> Option<policy::ReviewGate> {
        self.lifecycle.approval_gate()
    }

    /// Audit refs attached to this message.
    pub fn audit_refs(&self) -> &[crate::audit::EventId] {
        &self.audit_refs
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Entity or workflow subject that a message refers to.
pub enum MessageSubject {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Incident record participating in the workflow.
    Incident(IncidentId),
    /// Approval decision record participating in audit history.
    Approval(approval::Id),
}
