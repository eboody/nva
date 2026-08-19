use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::agents;

use domain::{agent, audit, customer, entities, message, pet, policy, workflow};

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
/// Decision choices for error in the daily update workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum Error {
    #[error("daily update preview could not build a validated domain value: {0}")]
    /// Identifies invalid domain value as the reason the workflow must stop, retry, or request review.
    InvalidDomainValue(String),
}

/// Result type returned by fallible daily update operations.
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Mvp preview used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
pub struct MvpPreview {
    /// Agent packet copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub agent_packet: agents::AgentPromptPacket<daily_care_update::Input>,
    /// Output copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub output: daily_care_update::Output,
    /// Owner message draft copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub owner_message_draft: CustomerMessageDraft,
    /// Approval copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub approval: entities::approval::Record,
    /// Send stub copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub send_stub: SendStub,
    /// Audit log copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub audit_log: Vec<audit::Event>,
}

/// Daily care notes prepared for staff review before they become operational updates.
pub mod daily_care_update {
    use serde::{Deserialize, Serialize};

    use super::{
        CustomerMessageDraft, IncludedFact, InternalFlag, MediaDocumentRef, OmittedFact,
        ReviewDisposition, SuppressedMediaDocumentRef, SuppressionRecord, customer, entities, pet,
        policy,
    };

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Input used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
    pub struct Input {
        /// Pet name copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub pet_name: pet::Name,
        /// Owner display name copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub owner_display_name: customer::Name,
        /// Policy snapshot id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub policy_snapshot_id: policy::Id,
        /// Notes copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub notes: Vec<entities::CareNote>,
        /// Media/document refs copied from the request so agents can see held media without publishing it.
        pub media_document_refs: Vec<MediaDocumentRef>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Output used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
    pub struct Output {
        /// Customer message copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub customer_message: CustomerMessageDraft,
        /// Internal flags copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub internal_flags: Vec<InternalFlag>,
        #[serde(flatten)]
        /// Disposition copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub disposition: ReviewDisposition,
        /// Included facts copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub included_facts: Vec<IncludedFact>,
        /// Omitted facts copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
        pub omitted_facts: Vec<OmittedFact>,
        /// Suppression records explaining why sensitive facts or source-ambiguous material stayed out of customer copy.
        pub suppression_records: Vec<SuppressionRecord>,
        /// Media/document refs held from the customer draft because they still require review.
        pub suppressed_media_document_refs: Vec<SuppressedMediaDocumentRef>,
    }

    #[derive(Debug, Clone, Copy)]
    /// Agent used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
    pub struct Agent;
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Decision choices for review disposition in the daily update workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum ReviewDisposition {
    /// Reason copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    DraftOnlyRequiresReview {
        /// Reason value stored on this variant.
        reason: ReviewReason,
    },
}

impl ReviewDisposition {
    /// Returns the allows live send evidence available to daily update review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn allows_live_send(&self) -> bool {
        false
    }

    /// Reports whether the daily update workflow satisfies the requires human review safety condition.
    pub const fn requires_human_review(&self) -> bool {
        true
    }

    /// Returns the review reason evidence available to daily update review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn review_reason(&self) -> &ReviewReason {
        match self {
            Self::DraftOnlyRequiresReview { reason } => reason,
        }
    }
}

impl Serialize for ReviewDisposition {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wire<'a> {
            should_send: bool,
            requires_review: bool,
            review_reason: &'a ReviewReason,
        }

        Wire {
            should_send: self.allows_live_send(),
            requires_review: self.requires_human_review(),
            review_reason: self.review_reason(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReviewDisposition {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            should_send: bool,
            requires_review: bool,
            review_reason: Option<ReviewReason>,
        }

        let wire = Wire::deserialize(deserializer)?;
        match (wire.should_send, wire.requires_review, wire.review_reason) {
            (false, true, Some(reason)) => Ok(Self::DraftOnlyRequiresReview { reason }),
            _ => Err(serde::de::Error::custom(
                "daily care update v1 output must remain a draft-only review-required disposition",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Customer message draft used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
pub struct CustomerMessageDraft {
    /// Body ref copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub body_ref: message::BodyRef,
    /// Channel hint copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub channel_hint: message::Channel,
    /// Language copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub language: LanguageTag,
    /// Tone copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub tone: ToneLabel,
    /// Audience copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub audience: Audience,
    /// Redaction profile copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub redaction_profile: RedactionProfile,
    /// Approved media/document refs allowed to accompany the customer draft after review.
    pub media_document_refs: Vec<MediaDocumentRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Media or document evidence reference considered for a Pawgress draft but never published until reviewed.
pub struct MediaDocumentRef {
    /// Document id that points reviewers to the photo/video/document evidence without embedding raw content.
    pub document_id: entities::DocumentId,
    /// Care note that proposed or justified the media/document use.
    pub source_note_id: entities::care_note::Id,
    /// Review state controlling whether this reference can appear in customer-facing draft output.
    pub review_state: message::ReviewState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Review-held media/document evidence with the policy reason it cannot be customer-visible yet.
pub struct SuppressedMediaDocumentRef {
    /// Proposed media/document reference kept out of the customer-facing draft.
    pub media_document_ref: MediaDocumentRef,
    /// Review reason that blocks publication or attachment.
    pub reason: message::SuppressionReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Suppression record that makes omitted daily-update facts auditable for staff review.
pub struct SuppressionRecord {
    /// Care note whose source fact was withheld from customer-facing copy.
    pub source_note_id: entities::care_note::Id,
    /// Shared message-safety reason for the suppression.
    pub reason: message::SuppressionReason,
    /// Human review gate that must clear before wording or send decisions proceed.
    pub required_gate: policy::ReviewGate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Decision choices for audience in the daily update workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum Audience {
    /// Selects customer for the daily update decision model so the app can choose a review, evidence, or draft path without taking live action.
    Customer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Decision choices for internal flag code in the daily update workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum InternalFlagCode {
    /// Uses customer message approval not configured as source-grounded evidence for the deterministic decision.
    CustomerMessageApprovalNotConfigured,
    /// Uses raw internal note not customer safe as source-grounded evidence for the deterministic decision.
    RawInternalNoteNotCustomerSafe,
    /// Uses behavior review required as source-grounded evidence for the deterministic decision.
    BehaviorReviewRequired,
    /// Uses medical or medication review required as source-grounded evidence for the deterministic decision.
    MedicalOrMedicationReviewRequired,
    /// Uses policy gap requires review as source-grounded evidence for the deterministic decision.
    PolicyGapRequiresReview,
    /// Uses payment or billing review required as source-grounded evidence for the deterministic decision.
    PaymentOrBillingReviewRequired,
    /// Uses incident or safety review required as source-grounded evidence for the deterministic decision.
    IncidentOrSafetyReviewRequired,
    /// Uses source ambiguity requires review as source-grounded evidence for the deterministic decision.
    SourceAmbiguityRequiresReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Decision choices for internal flag severity in the daily update workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum InternalFlagSeverity {
    /// Selects info for the daily update decision model so the app can choose a review, evidence, or draft path without taking live action.
    Info,
    /// Selects needs staff review for the daily update decision model so the app can choose a review, evidence, or draft path without taking live action.
    NeedsStaffReview,
    /// Selects needs manager review for the daily update decision model so the app can choose a review, evidence, or draft path without taking live action.
    NeedsManagerReview,
    /// Selects do not send for the daily update decision model so the app can choose a review, evidence, or draft path without taking live action.
    DoNotSend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Decision choices for recommended flag action in the daily update workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum RecommendedFlagAction {
    /// Selects staff review for the daily update decision model so the app can choose a review, evidence, or draft path without taking live action.
    StaffReview,
    /// Selects manager review for the daily update decision model so the app can choose a review, evidence, or draft path without taking live action.
    ManagerReview,
    /// Selects suppress update for the daily update decision model so the app can choose a review, evidence, or draft path without taking live action.
    SuppressUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Internal flag used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
pub struct InternalFlag {
    /// Code copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub code: InternalFlagCode,
    /// Severity copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub severity: InternalFlagSeverity,
    /// Message copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub message: FlagMessage,
    /// Source note ids copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub source_note_ids: Vec<entities::care_note::Id>,
    /// Recommended action copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub recommended_action: RecommendedFlagAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Included fact used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
pub struct IncludedFact {
    /// Source note id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub source_note_id: entities::care_note::Id,
    /// Summary copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub summary: FactSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Omitted fact used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
pub struct OmittedFact {
    /// Source note id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub source_note_id: entities::care_note::Id,
    /// Reason copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub reason: OmissionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Decision choices for omission reason in the daily update workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum OmissionReason {
    /// Uses internal only as source-grounded evidence for the deterministic decision.
    InternalOnly,
    /// Uses sensitive requires review as source-grounded evidence for the deterministic decision.
    SensitiveRequiresReview,
    /// Uses medical or medication review as source-grounded evidence for the deterministic decision.
    MedicalOrMedicationReview,
    /// Uses payment or billing review as source-grounded evidence for the deterministic decision.
    PaymentOrBillingReview,
    /// Uses incident or safety review as source-grounded evidence for the deterministic decision.
    IncidentOrSafetyReview,
    /// Uses source ambiguous review as source-grounded evidence for the deterministic decision.
    SourceAmbiguousReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Send stub used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
pub struct SendStub {
    /// Mode copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub mode: SendMode,
    /// Blocked by copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub blocked_by: Vec<policy::ReviewGate>,
    /// Audit action copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub audit_action: audit::Action,
}

impl SendStub {
    /// Reports whether the daily update workflow satisfies the is blocked until human approval safety condition.
    pub fn is_blocked_until_human_approval(&self) -> bool {
        matches!(self.mode, SendMode::ApprovalRequiredStub) && !self.blocked_by.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Decision choices for send mode in the daily update workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum SendMode {
    /// Selects approval required stub for the daily update decision model so the app can choose a review, evidence, or draft path without taking live action.
    ApprovalRequiredStub,
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
pub struct LanguageTag(String);

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
pub struct ToneLabel(String);

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
pub struct RedactionProfile(String);

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
pub struct ReviewReason(String);

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
pub struct FlagMessage(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
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
pub struct FactSummary(String);

impl agents::WorkflowAgent<daily_care_update::Input, daily_care_update::Output>
    for daily_care_update::Agent
{
    fn spec(&self) -> domain::agent::Spec {
        agents::baseline_agent_specs()
            .into_iter()
            .find(|spec| spec.name.clone().into_inner() == "daily-care-update")
            .expect("baseline daily-care-update agent spec exists")
    }

    fn build_prompt_packet(
        &self,
        event: &workflow::Event,
        input: daily_care_update::Input,
    ) -> agents::AgentPromptPacket<daily_care_update::Input> {
        agents::AgentPromptPacket::builder()
            .workflow_name(agent_name().expect("static daily-update agent name is valid"))
            .goal(agent::Purpose::try_new(
                "Transform source-backed staff care notes into a customer-safe draft preview while preserving approval gates and audit lineage.",
            ).expect("static daily-update purpose is valid"))
            .event(event.clone())
            .input(input)
            .policies(vec![agent::PolicyInstruction::try_new(
                "Draft only: live customer sends and health/behavior concern wording require human approval.",
            ).expect("static daily-update policy instruction is valid")])
            .output_schema_name(agent::OutputSchemaName::try_new("DailyCareUpdateOutput.v1").expect("static daily-update schema name is valid"))
            .build()
    }

    fn validate_output(
        &self,
        output: workflow::Result<daily_care_update::Output>,
    ) -> workflow::Result<daily_care_update::Output> {
        output
    }
}

fn agent_name() -> Result<agent::Name> {
    agent::Name::try_new("daily-care-update").map_err(invalid_domain_value)
}

fn invalid_domain_value(error: impl std::fmt::Display) -> Error {
    Error::InvalidDomainValue(error.to_string())
}
