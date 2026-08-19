use bon::Builder;
use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{access, entities, identity, policy, staff};

/// Permissioned assistant context and cited answer packets.
///
/// Canonical owner for assistant packet concepts previously introduced by
/// `strategic_ai_ops::assistant`. Answers are drafts/decision-support packets and must cite
/// approved knowledge or escalate; they do not authorize unsafe tools or live side effects.
pub mod assistant {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 4000),
        derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
    )]
    /// Assistant answer text.
    pub struct AnswerText(String);

    impl fmt::Debug for AnswerText {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("AnswerText([REDACTED])")
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Purpose of the assistant interaction.
    pub enum Purpose {
        /// SOP lookup.
        SopLookup,
        /// Pricing or service explanation.
        PricingQuestion,
        /// Vendor information lookup.
        VendorLookup,
        /// Site operational decision support.
        SiteOpsSupport,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Reason an assistant response should escalate.
    pub enum EscalationReason {
        /// Source conflict.
        ConflictingSources,
        /// Stale/missing source.
        StaleOrMissingSource,
        /// Sensitive customer, pet, financial, or safety context.
        SensitiveContext,
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Cited-answer lifecycle state.
    pub enum AnswerState {
        /// Draft output has not been proven against authorized evidence.
        #[default]
        Draft,
        /// Every claim is supported by an authorized retrieval passage.
        Cited,
        /// The assistant must route the question to review/escalation.
        Escalated,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Actor/location/purpose scope for permissioned assistant retrieval.
    pub struct ActorContext {
        actor_id: access::ActorId,
        role: access::ActorRole,
        title: access::Title,
        location_id: entities::LocationId,
        purpose: Purpose,
        #[builder(default)]
        allowed_uses: Vec<access::AllowedUse>,
    }

    impl ActorContext {
        /// Canonical resort location that scopes this assistant context.
        pub const fn location_id(&self) -> entities::LocationId {
            self.location_id
        }

        /// Actor role used for retrieval authorization.
        pub const fn role(&self) -> access::ActorRole {
            self.role
        }

        /// Request purpose used for retrieval authorization.
        pub const fn purpose(&self) -> Purpose {
            self.purpose
        }

        /// Returns whether the actor context declares the allowed use.
        pub fn allows_use(&self, allowed_use: access::AllowedUse) -> bool {
            self.allowed_uses.contains(&allowed_use)
        }

        /// Promotes this context into a canonical staff actor reference only for site labor roles.
        pub fn promote_staff_actor_ref(
            &self,
        ) -> std::result::Result<entities::ActorRef, ActorPromotionError> {
            let _: staff::Role = self
                .role
                .try_into()
                .map_err(ActorPromotionError::RoleNotStaffActor)?;
            if matches!(self.role, access::ActorRole::SiteManager) {
                return Err(ActorPromotionError::ManagerRoleIsNotStaffActor);
            }

            Ok(entities::ActorRef::Staff {
                staff_id: entities::StaffId::try_new(self.actor_id.as_ref()).expect(
                    "strategic actor id was already validated with staff-id-compatible rules",
                ),
            })
        }

        /// Promotes this context into a canonical manager actor reference only for site managers.
        pub fn promote_manager_actor_ref(
            &self,
        ) -> std::result::Result<entities::ActorRef, ActorPromotionError> {
            if !matches!(self.role, access::ActorRole::SiteManager) {
                return Err(ActorPromotionError::RoleNotManagerActor { role: self.role });
            }

            Ok(entities::ActorRef::Manager {
                manager_id: entities::ManagerId::try_new(self.actor_id.as_ref()).expect(
                    "strategic actor id was already validated with manager-id-compatible rules",
                ),
            })
        }
    }

    impl Purpose {
        /// Allowed-use authority required by this assistant request purpose.
        pub const fn required_allowed_use(self) -> access::AllowedUse {
            match self {
                Self::SopLookup
                | Self::PricingQuestion
                | Self::VendorLookup
                | Self::SiteOpsSupport => access::AllowedUse::InternalDecisionSupport,
            }
        }
    }

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
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
    /// Stable answer claim id used to bind generated claims to citations.
    pub struct ClaimId(String);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// One generated claim and its required supporting citation.
    pub struct Claim {
        id: ClaimId,
        citation: knowledge::Citation,
    }

    impl Claim {
        /// Citation that supports this claim.
        pub const fn citation(&self) -> &knowledge::Citation {
            &self.citation
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Failures returned when assistant context is promoted into canonical actor identity.
    pub enum ActorPromotionError {
        #[error("strategic access role is not a site staff actor")]
        /// The role cannot be used as a site staff actor.
        RoleNotStaffActor(#[from] access::RolePromotionError),
        #[error("site manager role must be promoted through manager actor identity")]
        /// Site manager identity is owned by manager ids, not staff ids.
        ManagerRoleIsNotStaffActor,
        #[error("strategic access role {role:?} is not a manager actor")]
        /// The role cannot be used as manager actor identity.
        RoleNotManagerActor {
            /// Actor role that cannot be treated as a manager.
            role: access::ActorRole,
        },
    }

    #[derive(Debug, PartialEq, Eq, bon::Builder)]
    /// Assistant answer with citations and escalation state.
    pub struct AnswerPacket {
        context: ActorContext,
        answer: AnswerText,
        #[builder(default)]
        state: AnswerState,
        #[builder(default)]
        claims: Vec<Claim>,
        #[builder(default)]
        authorized_evidence: Vec<knowledge::AuthorizedEvidence>,
        #[builder(default)]
        citations: Vec<knowledge::Citation>,
        confidence: identity::Confidence,
        escalation: Option<EscalationReason>,
    }

    impl AnswerPacket {
        /// Constructs a cited answer only when retrieval authorization preceded answer construction and every claim is supported.
        pub fn try_cited(
            context: ActorContext,
            answer: AnswerText,
            claims: Vec<Claim>,
            authorized_evidence: Vec<knowledge::AuthorizedEvidence>,
            citations: Vec<knowledge::Citation>,
            confidence: identity::Confidence,
        ) -> Result<Self> {
            if claims.is_empty() || citations.is_empty() {
                return Err(Error::UncitedClaim);
            }
            if authorized_evidence.is_empty() {
                return Err(Error::MissingAuthorizedEvidence);
            }
            if authorized_evidence
                .iter()
                .any(|evidence| evidence.has_source_conflict())
            {
                return Err(Error::ConflictingPolicyEvidence);
            }
            let all_claims_supported = claims.iter().all(|claim| {
                citations.contains(claim.citation())
                    && authorized_evidence
                        .iter()
                        .any(|evidence| evidence.supports(claim.citation()))
            });
            if !all_claims_supported {
                return Err(Error::UncitedClaim);
            }
            Ok(Self {
                context,
                answer,
                state: AnswerState::Cited,
                claims,
                authorized_evidence,
                citations,
                confidence,
                escalation: None,
            })
        }

        /// Constructs an escalated answer packet when cited authority cannot be established safely.
        pub fn escalated(
            context: ActorContext,
            answer: AnswerText,
            confidence: identity::Confidence,
            reason: EscalationReason,
        ) -> Self {
            Self {
                context,
                answer,
                state: AnswerState::Escalated,
                claims: Vec::new(),
                authorized_evidence: Vec::new(),
                citations: Vec::new(),
                confidence,
                escalation: Some(reason),
            }
        }

        /// Current cited-answer lifecycle state.
        pub const fn state(&self) -> AnswerState {
            self.state
        }

        /// Authorized evidence passages used to construct a cited answer.
        pub fn authorized_evidence(&self) -> &[knowledge::AuthorizedEvidence] {
            &self.authorized_evidence
        }

        /// Returns whether answer contains at least one citation and no missing-source escalation.
        pub fn is_cited(&self) -> bool {
            self.state == AnswerState::Cited
                && !self.citations.is_empty()
                && !self.claims.is_empty()
                && !self.authorized_evidence.is_empty()
                && self.escalation.is_none()
                && self.claims.iter().all(|claim| {
                    self.citations.contains(claim.citation())
                        && self
                            .authorized_evidence
                            .iter()
                            .any(|evidence| evidence.supports(claim.citation()))
                })
        }
    }

    impl Serialize for AnswerPacket {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeStruct as _;

            let mut packet = serializer.serialize_struct("AnswerPacket", 7)?;
            packet.serialize_field("context", &self.context)?;
            packet.serialize_field("answer", &self.answer)?;
            packet.serialize_field("state", &self.state)?;
            packet.serialize_field("claims", &self.claims)?;
            packet.serialize_field("citations", &self.citations)?;
            packet.serialize_field("confidence", &self.confidence)?;
            packet.serialize_field("escalation", &self.escalation)?;
            packet.end()
        }
    }

    impl<'de> Deserialize<'de> for AnswerPacket {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct RawAnswerPacket {
                context: ActorContext,
                answer: AnswerText,
                state: AnswerState,
                confidence: identity::Confidence,
                escalation: Option<EscalationReason>,
            }

            let raw = RawAnswerPacket::deserialize(deserializer)?;
            match raw.state {
                AnswerState::Cited => {
                    Err(serde::de::Error::custom(Error::MissingAuthorizedEvidence))
                }
                AnswerState::Escalated => {
                    let reason = raw
                        .escalation
                        .ok_or_else(|| serde::de::Error::custom(Error::MissingEscalationReason))?;
                    Ok(Self::escalated(
                        raw.context,
                        raw.answer,
                        raw.confidence,
                        reason,
                    ))
                }
                AnswerState::Draft => Ok(Self {
                    context: raw.context,
                    answer: raw.answer,
                    state: AnswerState::Draft,
                    claims: Vec::new(),
                    authorized_evidence: Vec::new(),
                    citations: Vec::new(),
                    confidence: raw.confidence,
                    escalation: None,
                }),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Assistant answer validation failures.
    pub enum Error {
        #[error("cited answer requires authorized retrieval evidence")]
        /// Cited state cannot be produced without authorized evidence.
        MissingAuthorizedEvidence,
        #[error("every answer claim must be supported by an authorized citation")]
        /// One or more generated claims lacked citation support.
        UncitedClaim,
        #[error("conflicting policy/source evidence must escalate instead of cite")]
        /// Source conflict prevents cited-answer promotion.
        ConflictingPolicyEvidence,
        #[error("escalated answer state requires an escalation reason")]
        /// Escalated state cannot be rehydrated without a reason.
        MissingEscalationReason,
    }

    /// Result type returned by answer constructors.
    pub type Result<T> = std::result::Result<T, Error>;
}

/// Approved knowledge metadata, applicability, and citations for assistant retrieval.
///
/// Canonical owner for permissioned knowledge concepts previously introduced by
/// `strategic_ai_ops::knowledge`. Knowledge facts must point to approved/fresh source material or
/// force escalation instead of becoming uncited answer authority.
pub mod knowledge {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
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
    /// Knowledge document id.
    pub struct DocumentId(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 240),
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
    /// Knowledge document title.
    pub struct Title(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 200),
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
    /// Section reference within a knowledge source.
    pub struct SectionRef(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 240),
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
    /// Stable retrieval passage id produced before answer construction.
    pub struct PassageId(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Kind of knowledge document.
    pub enum DocumentKind {
        /// SOP or operating procedure.
        Sop,
        /// Pricing/service sheet.
        Pricing,
        /// Vendor documentation or contract summary.
        Vendor,
        /// Training document.
        Training,
        /// Local site note approved for assistant use.
        SitePolicy,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Approval/freshness status for knowledge.
    pub enum ApprovalStatus {
        /// Approved for retrieval.
        Approved,
        /// Draft only.
        Draft,
        /// Stale and should be escalated.
        Stale,
        /// Superseded by a newer source.
        Superseded,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Applicability scope for a document.
    pub struct Applicability {
        #[builder(default)]
        locations: Vec<entities::LocationId>,
        #[builder(default)]
        services: Vec<entities::ServiceKind>,
        #[builder(default)]
        roles: Vec<access::ActorRole>,
    }

    impl Applicability {
        fn applies_to(
            &self,
            location_id: entities::LocationId,
            service: entities::ServiceKind,
            role: access::ActorRole,
        ) -> bool {
            self.locations.contains(&location_id)
                && self.services.contains(&service)
                && self.roles.contains(&role)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Approved knowledge document metadata for permissioned retrieval.
    pub struct Document {
        id: DocumentId,
        title: Title,
        kind: DocumentKind,
        status: ApprovalStatus,
        applicability: Applicability,
        #[builder(default)]
        sections: Vec<SectionRef>,
        effective_at: DateTime<Utc>,
        review_due_at: Option<DateTime<Utc>>,
    }

    impl Document {
        /// Document id used by citations.
        pub const fn id(&self) -> &DocumentId {
            &self.id
        }

        /// Returns whether this document applies to the actor/location/service context.
        pub fn applies_to(
            &self,
            location_id: entities::LocationId,
            service: entities::ServiceKind,
            role: access::ActorRole,
        ) -> bool {
            matches!(self.status, ApprovalStatus::Approved)
                && self.applicability.applies_to(location_id, service, role)
        }

        /// Returns whether this document is approved, effective, and not past its review deadline.
        pub fn is_current_at(&self, as_of: DateTime<Utc>) -> bool {
            matches!(self.status, ApprovalStatus::Approved)
                && self.effective_at <= as_of
                && self
                    .review_due_at
                    .is_none_or(|review_due_at| as_of < review_due_at)
        }

        /// Returns whether this document revision declares the cited section.
        pub fn contains_section(&self, section: &SectionRef) -> bool {
            self.sections.contains(section)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Cited source section used by assistant answers.
    pub struct Citation {
        document_id: DocumentId,
        section: SectionRef,
    }

    impl Citation {
        /// Cited document id.
        pub const fn document_id(&self) -> &DocumentId {
            &self.document_id
        }

        /// Cited section reference.
        pub const fn section(&self) -> &SectionRef {
            &self.section
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    /// Opaque retrieval passage issued only by a future authenticated actor/document root.
    ///
    /// No public constructor exists; serializable `ActorContext` and `Document` metadata cannot
    /// promote themselves into this accepted evidence type.
    pub struct AuthorizedEvidence {
        document_id: DocumentId,
        passage_id: PassageId,
        section: SectionRef,
        retrieved_at: DateTime<Utc>,
        allowed_use: access::AllowedUse,
        source_conflict: bool,
    }

    impl AuthorizedEvidence {
        /// Returns whether this internally issued passage supports the citation exactly.
        pub(super) fn supports(&self, citation: &Citation) -> bool {
            self.document_id == *citation.document_id() && self.section == *citation.section()
        }

        /// Whether this passage carries conflicting policy/source evidence.
        pub(super) const fn has_source_conflict(&self) -> bool {
            self.source_conflict
        }

        #[cfg(test)]
        pub(super) fn test_only(
            document_id: DocumentId,
            section: SectionRef,
            source_conflict: bool,
        ) -> Self {
            Self {
                document_id,
                passage_id: PassageId::try_new("test-passage").unwrap(),
                section,
                retrieved_at: DateTime::<Utc>::UNIX_EPOCH,
                allowed_use: access::AllowedUse::InternalDecisionSupport,
                source_conflict,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Knowledge retrieval authorization failures.
    pub enum Error {
        #[error("knowledge document is not approved, effective, and current for retrieval")]
        /// Draft, stale, superseded, not-yet-effective, or review-expired documents cannot authorize citations.
        DocumentNotApprovedOrCurrent,
        #[error("knowledge document does not apply to actor role, location, or service")]
        /// Document applicability did not match request role/location/service.
        ApplicabilityMismatch,
        #[error("knowledge document does not contain the cited section")]
        /// Retrieved section was not declared on the document revision.
        SectionNotFound,
        #[error("actor request purpose is not authorized for knowledge retrieval")]
        /// Actor context lacks the allowed use required by the request purpose.
        PurposeNotAuthorized,
    }

    /// Result type returned by knowledge retrieval authorization.
    pub type Result<T> = std::result::Result<T, Error>;
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
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
pub struct Name(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 400),
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
pub struct Purpose(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
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
pub struct ToolName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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
pub struct ForbiddenAction(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 400),
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
pub struct PolicyInstruction(String);

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
pub struct OutputSchemaName(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Domain contract for one bounded automation agent.
///
/// A spec describes what an agent is meant to help with, which narrow tools may
/// be exposed to it, which live operational actions are outside its authority,
/// and which deterministic review gates must remain in the app workflow.
pub struct Spec {
    /// Stable workflow-facing agent name.
    ///
    /// This is an identifier such as `manager-daily-brief` or `booking-triage`,
    /// not a human display label. It connects prompt packets, outputs, and audit
    /// evidence back to the spec that constrained the agent run.
    pub name: Name,
    /// Business purpose for the agent's draft or evidence work.
    ///
    /// The purpose should state the resort operation being supported, such as
    /// summarizing labor risk, drafting customer-safe follow-up, or routing
    /// vaccine-document ambiguity; it does not grant live-action authority.
    pub purpose: Purpose,
    /// Tool names the runtime may expose to this agent.
    ///
    /// These should be read-only, draft-only, or task-creation surfaces scoped to
    /// the workflow. They are the positive capability list for context building,
    /// not permission to bypass review gates or mutate source systems.
    pub allowed_tools: Vec<ToolName>,
    /// Live or unsafe actions the agent must not perform directly.
    ///
    /// Examples include confirming bookings, promising availability, changing
    /// labor schedules, waiving deposits, diagnosing pets, or sending customer
    /// messages without approval.
    pub forbidden_actions: Vec<ForbiddenAction>,
    /// Human or deterministic app review gates required for the workflow.
    ///
    /// These gates keep manager approval, customer-message approval, medical
    /// document review, and similar authority outside the model-generated output.
    pub default_review_gates: Vec<policy::ReviewGate>,
}

#[cfg(test)]
mod coverage_convergence_tests {
    use chrono::TimeZone as _;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn context(role: access::ActorRole) -> assistant::ActorContext {
        assistant::ActorContext::builder()
            .actor_id(access::ActorId::try_new("actor-7").unwrap())
            .role(role)
            .title(access::Title::try_new("shift lead").unwrap())
            .location_id(entities::LocationId::new(Uuid::from_u128(7)))
            .purpose(assistant::Purpose::SopLookup)
            .allowed_uses(vec![access::AllowedUse::InternalDecisionSupport])
            .build()
    }

    #[test]
    fn assistant_identity_promotion_and_purpose_scopes_remain_exact() {
        let staff = context(access::ActorRole::FrontDesk);
        assert_eq!(
            staff.location_id(),
            entities::LocationId::new(Uuid::from_u128(7))
        );
        assert_eq!(staff.role(), access::ActorRole::FrontDesk);
        assert_eq!(staff.purpose(), assistant::Purpose::SopLookup);
        assert!(staff.allows_use(access::AllowedUse::InternalDecisionSupport));
        assert!(matches!(
            staff.promote_staff_actor_ref(),
            Ok(entities::ActorRef::Staff { .. })
        ));
        assert!(matches!(
            staff.promote_manager_actor_ref(),
            Err(assistant::ActorPromotionError::RoleNotManagerActor { .. })
        ));

        let manager = context(access::ActorRole::SiteManager);
        assert!(matches!(
            manager.promote_manager_actor_ref(),
            Ok(entities::ActorRef::Manager { .. })
        ));
        assert_eq!(
            manager.promote_staff_actor_ref(),
            Err(assistant::ActorPromotionError::ManagerRoleIsNotStaffActor)
        );
        let regional = context(access::ActorRole::RegionalOperations);
        assert!(matches!(
            regional.promote_staff_actor_ref(),
            Err(assistant::ActorPromotionError::RoleNotStaffActor(_))
        ));
        for purpose in [
            assistant::Purpose::SopLookup,
            assistant::Purpose::PricingQuestion,
            assistant::Purpose::VendorLookup,
            assistant::Purpose::SiteOpsSupport,
        ] {
            assert_eq!(
                purpose.required_allowed_use(),
                access::AllowedUse::InternalDecisionSupport
            );
        }
    }

    #[test]
    fn cited_answers_require_exact_nonconflicting_authorized_passages() {
        let document_id = knowledge::DocumentId::try_new("sop-7").unwrap();
        let section = knowledge::SectionRef::try_new("check-in").unwrap();
        let citation = knowledge::Citation::builder()
            .document_id(document_id.clone())
            .section(section.clone())
            .build();
        let claim = assistant::Claim::builder()
            .id(assistant::ClaimId::try_new("claim-7").unwrap())
            .citation(citation.clone())
            .build();
        let evidence =
            knowledge::AuthorizedEvidence::test_only(document_id.clone(), section.clone(), false);
        let packet = assistant::AnswerPacket::try_cited(
            context(access::ActorRole::FrontDesk),
            assistant::AnswerText::try_new("Use the approved check-in SOP.").unwrap(),
            vec![claim],
            vec![evidence],
            vec![citation.clone()],
            identity::Confidence::High,
        )
        .unwrap();
        assert_eq!(packet.state(), assistant::AnswerState::Cited);
        assert!(packet.is_cited());
        assert_eq!(packet.authorized_evidence().len(), 1);
        let encoded = serde_json::to_value(&packet).unwrap();
        assert!(encoded.get("authorized_evidence").is_none());

        let conflicting =
            knowledge::AuthorizedEvidence::test_only(document_id.clone(), section.clone(), true);
        assert_eq!(
            assistant::AnswerPacket::try_cited(
                context(access::ActorRole::FrontDesk),
                assistant::AnswerText::try_new("conflict").unwrap(),
                vec![
                    assistant::Claim::builder()
                        .id(assistant::ClaimId::try_new("claim-conflict").unwrap())
                        .citation(citation.clone())
                        .build()
                ],
                vec![conflicting],
                vec![citation.clone()],
                identity::Confidence::Low,
            ),
            Err(assistant::Error::ConflictingPolicyEvidence)
        );
        let unsupported = knowledge::AuthorizedEvidence::test_only(
            document_id,
            knowledge::SectionRef::try_new("different-section").unwrap(),
            false,
        );
        assert_eq!(
            assistant::AnswerPacket::try_cited(
                context(access::ActorRole::FrontDesk),
                assistant::AnswerText::try_new("unsupported").unwrap(),
                vec![
                    assistant::Claim::builder()
                        .id(assistant::ClaimId::try_new("claim-unsupported").unwrap())
                        .citation(citation.clone())
                        .build()
                ],
                vec![unsupported],
                vec![citation],
                identity::Confidence::Low,
            ),
            Err(assistant::Error::UncitedClaim)
        );
    }

    #[test]
    fn serialized_answer_state_cannot_rehydrate_cited_authority() {
        assert!(
            serde_json::from_value::<assistant::AnswerPacket>(json!({
                "context": context(access::ActorRole::FrontDesk),
                "answer": "caller supplied",
                "state": "Cited",
                "confidence": "High",
                "escalation": null
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<assistant::AnswerPacket>(json!({
                "context": context(access::ActorRole::FrontDesk),
                "answer": "caller supplied",
                "state": "Escalated",
                "confidence": "Low",
                "escalation": null
            }))
            .is_err()
        );
        let draft: assistant::AnswerPacket = serde_json::from_value(json!({
            "context": context(access::ActorRole::FrontDesk),
            "answer": "draft only",
            "state": "Draft",
            "confidence": "Low",
            "escalation": null
        }))
        .unwrap();
        assert_eq!(draft.state(), assistant::AnswerState::Draft);
        assert!(!draft.is_cited());
        assert_eq!(
            format!(
                "{:?}",
                assistant::AnswerText::try_new("private answer").unwrap()
            ),
            "AnswerText([REDACTED])"
        );
    }

    #[test]
    fn approved_knowledge_checks_applicability_currency_and_declared_sections() {
        let now = Utc.with_ymd_and_hms(2026, 8, 18, 5, 0, 0).unwrap();
        let section = knowledge::SectionRef::try_new("check-in").unwrap();
        let document = knowledge::Document::builder()
            .id(knowledge::DocumentId::try_new("sop-7").unwrap())
            .title(knowledge::Title::try_new("Check-in SOP").unwrap())
            .kind(knowledge::DocumentKind::Sop)
            .status(knowledge::ApprovalStatus::Approved)
            .applicability(
                knowledge::Applicability::builder()
                    .locations(vec![entities::LocationId::new(Uuid::from_u128(7))])
                    .services(vec![entities::ServiceKind::Boarding])
                    .roles(vec![access::ActorRole::FrontDesk])
                    .build(),
            )
            .sections(vec![section.clone()])
            .effective_at(now - chrono::Duration::hours(1))
            .review_due_at(now + chrono::Duration::hours(1))
            .build();
        assert_eq!(
            document.id(),
            &knowledge::DocumentId::try_new("sop-7").unwrap()
        );
        assert!(document.applies_to(
            entities::LocationId::new(Uuid::from_u128(7)),
            entities::ServiceKind::Boarding,
            access::ActorRole::FrontDesk
        ));
        assert!(document.is_current_at(now));
        assert!(document.contains_section(&section));
    }
}
