use bon::Builder;
use chrono::{DateTime, Utc};
use domain::{agent, entities, identity};
use std::fmt;

/// Marker seam for a future authenticated knowledge repository.
///
/// Implementations expose no accepted document data until an actor/document authority root exists.
pub trait Repository {}

#[derive(Debug, Default, Clone, Copy)]
/// Empty deterministic repository proving that fixture metadata cannot issue retrieval authority.
pub struct DeterministicFixtureRepository;

impl DeterministicFixtureRepository {
    /// Compatibility constructor for an unaccepted conflicting-source fixture scenario.
    pub const fn with_conflicting_boarding_fixture() -> Self {
        Self
    }
}

impl Repository for DeterministicFixtureRepository {}

#[derive(Debug, Clone, Builder)]
/// Actor-scoped knowledge retrieval request.
pub struct Request {
    context: agent::assistant::ActorContext,
    service: entities::ServiceKind,
    requested_section: agent::knowledge::SectionRef,
    requested_at: DateTime<Utc>,
}

/// Redacted app/API assistant packet for a permissioned knowledge answer.
pub struct Packet {
    answer: agent::assistant::AnswerPacket,
    authorized_passages: Vec<agent::knowledge::AuthorizedEvidence>,
    citations: Vec<agent::knowledge::Citation>,
    escalation_reason: Option<agent::assistant::EscalationReason>,
    forbidden_actions: Vec<&'static str>,
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packet")
            .field("answer_state", &self.answer_state())
            .field("authorized_passage_count", &self.authorized_passages.len())
            .field("citation_count", &self.citations.len())
            .field("escalation_reason", &self.escalation_reason)
            .field("forbidden_actions", &self.forbidden_actions)
            .finish()
    }
}

impl Packet {
    /// Returns whether this packet may be inserted into assistant context.
    pub fn safe_to_enter_assistant_context(&self) -> bool {
        self.answer.is_cited()
            && !self.authorized_passages.is_empty()
            && self.escalation_reason.is_none()
    }

    /// Current answer state.
    pub fn answer_state(&self) -> agent::assistant::AnswerState {
        self.answer.state()
    }

    /// Authorized passages used to construct this packet.
    pub fn authorized_passages(&self) -> &[agent::knowledge::AuthorizedEvidence] {
        &self.authorized_passages
    }

    /// Citations exposed with this packet.
    pub fn citations(&self) -> &[agent::knowledge::Citation] {
        &self.citations
    }

    /// Whether the answer is cited and all claims were supported before context entry.
    pub fn claims_are_cited(&self) -> bool {
        self.answer.is_cited()
    }

    /// Unsafe or live operational actions unavailable to this assistant packet.
    pub fn forbidden_actions(&self) -> &[&'static str] {
        &self.forbidden_actions
    }

    /// Whether this packet can perform live side effects.
    pub const fn live_side_effects_allowed(&self) -> bool {
        false
    }

    /// Escalation reason when the packet cannot cite safely.
    pub const fn escalation_reason(&self) -> Option<agent::assistant::EscalationReason> {
        self.escalation_reason
    }
}

/// Permissioned retrieval workflow that authorizes before answer construction.
pub struct Workflow;

impl Workflow {
    /// Fails closed until authenticated actor and document-approval roots can issue opaque access.
    pub fn answer(_repository: &impl Repository, request: Request) -> Packet {
        let forbidden_actions = vec![
            "send_customer_message",
            "provider_pms_write",
            "schedule_capacity_mutation",
            "refund_discount_payment_action",
        ];
        let _untrusted_request_evidence = (
            &request.service,
            &request.requested_section,
            request.requested_at,
        );
        escalated_packet(
            request.context,
            agent::assistant::EscalationReason::StaleOrMissingSource,
            forbidden_actions,
        )
    }
}

fn escalated_packet(
    context: agent::assistant::ActorContext,
    reason: agent::assistant::EscalationReason,
    forbidden_actions: Vec<&'static str>,
) -> Packet {
    let answer = agent::assistant::AnswerPacket::escalated(
        context,
        agent::assistant::AnswerText::try_new(
            "Escalate this knowledge request because authorized source evidence is unavailable.",
        )
        .expect("static escalation answer is valid"),
        identity::Confidence::Medium,
        reason,
    );
    Packet {
        answer,
        authorized_passages: Vec::new(),
        citations: Vec::new(),
        escalation_reason: Some(reason),
        forbidden_actions,
    }
}
