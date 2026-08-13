use bon::Builder;
use chrono::{DateTime, TimeZone, Utc};
use domain::{access, agent, entities, identity};
use std::fmt;

/// Repository seam for approved knowledge documents used by permissioned assistant retrieval.
///
/// Implementations must return fixture or durable document metadata only; they must not expose
/// raw provider payloads, customer messages, PMS writes, or source-system mutation tools.
pub trait Repository {
    /// Returns document metadata applicable to the request candidate set.
    fn documents(&self) -> Vec<KnowledgeDocumentFixture>;
}

#[derive(Clone)]
/// Deterministic fixture repository for local knowledge-retrieval proof.
pub struct DeterministicFixtureRepository {
    documents: Vec<KnowledgeDocumentFixture>,
}

impl Default for DeterministicFixtureRepository {
    fn default() -> Self {
        Self {
            documents: vec![KnowledgeDocumentFixture::boarding_check_in_sop(false)],
        }
    }
}

impl DeterministicFixtureRepository {
    /// Returns a fixture repository where the otherwise applicable SOP has source conflicts.
    pub fn with_conflicting_boarding_fixture() -> Self {
        Self {
            documents: vec![KnowledgeDocumentFixture::boarding_check_in_sop(true)],
        }
    }
}

impl Repository for DeterministicFixtureRepository {
    fn documents(&self) -> Vec<KnowledgeDocumentFixture> {
        self.documents.clone()
    }
}

#[derive(Clone)]
/// Fixture document plus deterministic passage/claim payload used by the app seam.
pub struct KnowledgeDocumentFixture {
    document: agent::knowledge::Document,
    passage_id: agent::knowledge::PassageId,
    section: agent::knowledge::SectionRef,
    answer: agent::assistant::AnswerText,
    claim_id: agent::assistant::ClaimId,
    source_conflict: bool,
}

impl KnowledgeDocumentFixture {
    fn boarding_check_in_sop(source_conflict: bool) -> Self {
        let section = agent::knowledge::SectionRef::try_new("check-in.required-documents")
            .expect("static fixture section is valid");
        Self {
            document: agent::knowledge::Document::builder()
                .id(agent::knowledge::DocumentId::try_new("sop-boarding-v1").unwrap())
                .title(agent::knowledge::Title::try_new("Boarding check-in SOP").unwrap())
                .kind(agent::knowledge::DocumentKind::Sop)
                .status(agent::knowledge::ApprovalStatus::Approved)
                .applicability(
                    agent::knowledge::Applicability::builder()
                        .locations(vec![entities::LocationId(uuid::Uuid::from_u128(0x170))])
                        .services(vec![entities::ServiceKind::Boarding])
                        .roles(vec![access::ActorRole::FrontDesk])
                        .build(),
                )
                .sections(vec![section.clone()])
                .effective_at(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap())
                .review_due_at(Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap())
                .build(),
            passage_id: agent::knowledge::PassageId::try_new(
                "retrieval-passages/sop-boarding-v1#check-in.required-documents",
            )
            .unwrap(),
            section,
            answer: agent::assistant::AnswerText::try_new(
                "Use the boarding check-in checklist and route vaccine ambiguity to manager review.",
            )
            .unwrap(),
            claim_id: agent::assistant::ClaimId::try_new("claim-checkin-required-documents")
                .unwrap(),
            source_conflict,
        }
    }
}

#[derive(Debug, Clone, Builder)]
/// Actor-scoped knowledge retrieval request.
pub struct Request {
    context: agent::assistant::ActorContext,
    service: entities::ServiceKind,
    requested_section: agent::knowledge::SectionRef,
    requested_at: DateTime<Utc>,
}

#[derive(Clone)]
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
    /// Builds a cited answer only from authorized fixture repository evidence, otherwise escalates.
    pub fn answer(repository: &impl Repository, request: Request) -> Packet {
        let forbidden_actions = vec![
            "send_customer_message",
            "provider_pms_write",
            "schedule_capacity_mutation",
            "refund_discount_payment_action",
        ];

        let Some(fixture) = repository.documents().into_iter().find(|fixture| {
            fixture.section == request.requested_section
                && fixture.document.applies_to(
                    request.context.location_id(),
                    request.service.clone(),
                    request.context.role(),
                )
        }) else {
            return escalated_packet(
                request.context,
                agent::assistant::EscalationReason::StaleOrMissingSource,
                forbidden_actions,
            );
        };

        let Ok(mut evidence) = agent::knowledge::AuthorizedEvidence::try_from_retrieval(
            &fixture.document,
            fixture.passage_id,
            fixture.section.clone(),
            request.requested_at,
            &request.context,
            request.service,
        ) else {
            return escalated_packet(
                request.context,
                agent::assistant::EscalationReason::StaleOrMissingSource,
                forbidden_actions,
            );
        };

        if fixture.source_conflict {
            evidence = evidence.with_source_conflict();
        }

        let citation = agent::knowledge::Citation::builder()
            .document_id(fixture.document.id().clone())
            .section(fixture.section)
            .build();
        let claim = agent::assistant::Claim::builder()
            .id(fixture.claim_id)
            .citation(citation.clone())
            .build();

        match agent::assistant::AnswerPacket::try_cited(
            request.context.clone(),
            fixture.answer,
            vec![claim],
            vec![evidence.clone()],
            vec![citation.clone()],
            identity::Confidence::High,
        ) {
            Ok(answer) => Packet {
                answer,
                authorized_passages: vec![evidence],
                citations: vec![citation],
                escalation_reason: None,
                forbidden_actions,
            },
            Err(agent::assistant::Error::ConflictingPolicyEvidence) => escalated_packet(
                request.context,
                agent::assistant::EscalationReason::ConflictingSources,
                forbidden_actions,
            ),
            Err(_) => escalated_packet(
                request.context,
                agent::assistant::EscalationReason::StaleOrMissingSource,
                forbidden_actions,
            ),
        }
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
