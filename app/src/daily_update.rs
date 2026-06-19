use bon::Builder;
use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::agents;
use crate::agents::WorkflowAgent;
use domain::{agent, audit, customer, entities, message, pet, policy, workflow};

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
/// Decision choices for error in the daily update workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum Error {
    #[error("daily update preview requires a DailyNoteCreated or DailyUpdateNeeded workflow event")]
    /// Identifies unsupported workflow event as the reason the workflow must stop, retry, or request review.
    UnsupportedWorkflowEvent,
    #[error("daily update preview requires at least one staff note")]
    /// Identifies missing staff notes as the reason the workflow must stop, retry, or request review.
    MissingStaffNotes,
    #[error("daily update preview requires at least one policy-allowed draft/summarize action")]
    /// Identifies missing allowed action as the reason the workflow must stop, retry, or request review.
    MissingAllowedAction,
    #[error("daily update preview could not build a validated domain value: {0}")]
    /// Identifies invalid domain value as the reason the workflow must stop, retry, or request review.
    InvalidDomainValue(String),
}

/// Result type returned by fallible daily update operations.
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Mvp preview request used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
pub struct MvpPreviewRequest {
    /// Event copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub event: workflow::Event,
    /// Pet name copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub pet_name: pet::Name,
    /// Owner display name copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub owner_display_name: customer::Name,
    /// Policy snapshot id copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub policy_snapshot_id: policy::Id,
    /// Notes copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub notes: Vec<entities::CareNote>,
}

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
    pub audit_log: Vec<entities::audit::Event>,
}

/// Daily care notes prepared for staff review before they become operational updates.
pub mod daily_care_update {
    use serde::{Deserialize, Serialize};

    use super::{
        CustomerMessageDraft, IncludedFact, InternalFlag, OmittedFact, ReviewDisposition, customer,
        entities, pet, policy,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Send stub used by the daily update workflow; it packages operational changes into reviewable staff updates instead of free-form agent output.
pub struct SendStub {
    /// Mode copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub mode: SendMode,
    /// Blocked by copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub blocked_by: Vec<policy::ReviewGate>,
    /// Audit action copied from reviewed source input for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    pub audit_action: entities::audit::Action,
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

/// Builds the mvp preview output for the daily update workflow.
pub fn build_mvp_preview(request: MvpPreviewRequest) -> Result<MvpPreview> {
    validate_request(&request)?;

    let input = daily_care_update::Input {
        pet_name: request.pet_name,
        owner_display_name: request.owner_display_name,
        policy_snapshot_id: request.policy_snapshot_id,
        notes: request.notes,
    };

    let agent = daily_care_update::Agent;
    let agent_packet = agent.build_prompt_packet(&request.event, input.clone());
    let output = generate_output(&input)?;
    let review_gate = review_gate_for(&output);

    let message_id =
        entities::MessageId(Uuid::from_u128(0xDA17_0000_0000_0000_0000_0000_0000_0001));
    let approval_id =
        entities::approval::Id(Uuid::from_u128(0xDA17_0000_0000_0000_0000_0000_0000_0002));
    let _owner_message_record = entities::Message::builder()
        .id(message_id)
        .subject(entities::MessageSubject::Reservation(
            subject_reservation_id(&request.event),
        ))
        .direction(message::Direction::OutboundDraft)
        .channel(message::Channel::Portal)
        .status(message::Status::ApprovalRequested)
        .body_ref(output.customer_message.body_ref.clone())
        .approval_gate(review_gate.clone())
        .audit_refs(vec![audit::EventId(Uuid::from_u128(
            0xDA17_0000_0000_0000_0000_0000_0000_0003,
        ))])
        .build();

    let approval = entities::approval::Record::builder()
        .id(approval_id)
        .target(entities::approval::Target::Message(message_id))
        .gate(review_gate.clone())
        .lifecycle(entities::approval::Lifecycle::ApprovalRequested)
        .requested_by(entities::ActorRef::Agent {
            workflow: agent_name()?,
        })
        .requested_at(request.event.occurred_at)
        .audit_refs(vec![audit::EventId(Uuid::from_u128(
            0xDA17_0000_0000_0000_0000_0000_0000_0004,
        ))])
        .build();

    let send_stub = SendStub {
        mode: SendMode::ApprovalRequiredStub,
        blocked_by: vec![review_gate],
        audit_action: entities::audit::Action::Extension(audit_action_label(
            "message.send.blocked_stub",
        )?),
    };

    let audit_log = audit_log(&request.event, message_id, approval_id)?;
    let owner_message_draft = output.customer_message.clone();

    Ok(MvpPreview {
        agent_packet,
        owner_message_draft,
        output,
        approval,
        send_stub,
        audit_log,
    })
}

impl agents::WorkflowAgent<daily_care_update::Input, daily_care_update::Output>
    for daily_care_update::Agent
{
    fn spec(&self) -> agents::AgentSpec {
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

fn validate_request(request: &MvpPreviewRequest) -> Result<()> {
    if !matches!(
        request.event.event_type,
        workflow::EventType::DailyNoteCreated | workflow::EventType::DailyUpdateNeeded
    ) {
        return Err(Error::UnsupportedWorkflowEvent);
    }
    if request.notes.is_empty() {
        return Err(Error::MissingStaffNotes);
    }
    if !request
        .event
        .policy_context
        .allowed_actions
        .iter()
        .any(|action| {
            matches!(
                action,
                workflow::AllowedAction::SummarizeCareNotes
                    | workflow::AllowedAction::DraftCustomerMessage
            )
        })
    {
        return Err(Error::MissingAllowedAction);
    }
    Ok(())
}

fn generate_output(input: &daily_care_update::Input) -> Result<daily_care_update::Output> {
    let mut included_facts = Vec::new();
    let mut omitted_facts = Vec::new();
    let mut internal_flags = vec![InternalFlag {
        code: InternalFlagCode::CustomerMessageApprovalNotConfigured,
        severity: InternalFlagSeverity::NeedsStaffReview,
        message: flag_message(
            "Daily care updates are draft-only until a location/channel/template send policy is approved.",
        )?,
        source_note_ids: input.notes.iter().map(|note| note.id).collect(),
        recommended_action: RecommendedFlagAction::StaffReview,
    }];

    let mut safe_note_bodies = Vec::new();
    let mut sensitive_note_ids = Vec::new();

    for note in &input.notes {
        if matches!(
            note.visibility,
            entities::care_note::Visibility::InternalOnly
        ) {
            omitted_facts.push(OmittedFact {
                source_note_id: note.id,
                reason: OmissionReason::InternalOnly,
            });
            internal_flags.push(InternalFlag {
                code: InternalFlagCode::RawInternalNoteNotCustomerSafe,
                severity: InternalFlagSeverity::DoNotSend,
                message: flag_message("Raw internal staff notes are omitted from customer copy.")?,
                source_note_ids: vec![note.id],
                recommended_action: RecommendedFlagAction::SuppressUpdate,
            });
            continue;
        }

        if sensitive_kind(note.kind) {
            omitted_facts.push(OmittedFact {
                source_note_id: note.id,
                reason: OmissionReason::SensitiveRequiresReview,
            });
            sensitive_note_ids.push(note.id);
            continue;
        }

        let summary = normalize_sentence(note.body.clone().into_inner());
        included_facts.push(IncludedFact {
            source_note_id: note.id,
            summary: FactSummary::try_new(summary.clone()).map_err(invalid_domain_value)?,
        });
        safe_note_bodies.push(summary);
    }

    let sensitive_review = if !sensitive_note_ids.is_empty() {
        let code = if input
            .notes
            .iter()
            .any(|note| matches!(note.kind, entities::care_note::Kind::Behavior))
        {
            InternalFlagCode::BehaviorReviewRequired
        } else {
            InternalFlagCode::MedicalOrMedicationReviewRequired
        };
        internal_flags.push(InternalFlag {
            code,
            severity: InternalFlagSeverity::NeedsManagerReview,
            message: flag_message("Sensitive care-note content was suppressed until manager review approves customer wording.")?,
            source_note_ids: sensitive_note_ids,
            recommended_action: RecommendedFlagAction::ManagerReview,
        });
        Some(code)
    } else {
        None
    };

    let review_reason = match sensitive_review {
        Some(InternalFlagCode::BehaviorReviewRequired) => "behavior_review_required",
        Some(_) => "medical_or_medication_review_required",
        None => "customer_message_approval_not_configured",
    };

    let body = if safe_note_bodies.is_empty() {
        format!(
            "Hi {} — {}'s daily update is being reviewed by our care team before we share customer-facing wording.",
            input.owner_display_name.clone().into_inner(),
            input.pet_name.clone().into_inner()
        )
    } else {
        format!(
            "Hi {} — {} {}",
            input.owner_display_name.clone().into_inner(),
            input.pet_name.clone().into_inner(),
            safe_note_bodies.join(" ")
        )
    };

    Ok(daily_care_update::Output {
        customer_message: CustomerMessageDraft {
            body_ref: message::BodyRef::try_new(body).map_err(invalid_domain_value)?,
            channel_hint: message::Channel::Portal,
            language: LanguageTag::try_new("en-US").map_err(invalid_domain_value)?,
            tone: ToneLabel::try_new("warm_concise_factual").map_err(invalid_domain_value)?,
            audience: Audience::Customer,
            redaction_profile: RedactionProfile::try_new("customer_safe_daily_update_v1")
                .map_err(invalid_domain_value)?,
        },
        internal_flags,
        disposition: ReviewDisposition::DraftOnlyRequiresReview {
            reason: ReviewReason::try_new(review_reason).map_err(invalid_domain_value)?,
        },
        included_facts,
        omitted_facts,
    })
}

fn review_gate_for(output: &daily_care_update::Output) -> policy::ReviewGate {
    if output.internal_flags.iter().any(|flag| {
        matches!(
            flag.code,
            InternalFlagCode::BehaviorReviewRequired
                | InternalFlagCode::MedicalOrMedicationReviewRequired
        )
    }) {
        policy::ReviewGate::ManagerApproval
    } else {
        policy::ReviewGate::CustomerMessageApproval
    }
}

fn audit_log(
    event: &workflow::Event,
    message_id: entities::MessageId,
    approval_id: entities::approval::Id,
) -> Result<Vec<entities::audit::Event>> {
    Ok(vec![
        audit_event(
            event.occurred_at,
            event.actor.clone(),
            entities::audit::Subject::WorkflowEvent(event.event_id),
            entities::audit::Action::WorkflowEventRecorded,
            "daily-care-update workflow event recorded for MVP preview",
        )?,
        audit_event(
            event.occurred_at,
            entities::ActorRef::Agent {
                workflow: agent_name()?,
            },
            entities::audit::Subject::Message(message_id),
            entities::audit::Action::MessageApprovalRequested,
            "daily care update owner-message draft created; no live send attempted",
        )?,
        audit_event(
            event.occurred_at,
            entities::ActorRef::Agent {
                workflow: agent_name()?,
            },
            entities::audit::Subject::Approval(approval_id),
            entities::audit::Action::ApprovalDecisionRecorded,
            "approval record opened for staff/manager review stub",
        )?,
    ])
}

fn audit_event(
    at: DateTime<Utc>,
    actor: entities::ActorRef,
    subject: entities::audit::Subject,
    action: entities::audit::Action,
    summary: &str,
) -> Result<entities::audit::Event> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        entities::audit::MetadataKey::try_new("summary").map_err(invalid_domain_value)?,
        entities::audit::MetadataValue::try_new(summary).map_err(invalid_domain_value)?,
    );
    Ok(entities::audit::Event {
        at,
        actor,
        subject,
        action,
        metadata,
    })
}

fn subject_reservation_id(event: &workflow::Event) -> entities::reservation::Id {
    match event.subject {
        workflow::Subject::Reservation(id) => id,
        _ => entities::reservation::Id(Uuid::nil()),
    }
}

fn sensitive_kind(kind: entities::care_note::Kind) -> bool {
    matches!(
        kind,
        entities::care_note::Kind::Medication
            | entities::care_note::Kind::Medical
            | entities::care_note::Kind::Behavior
    )
}

fn normalize_sentence(body: String) -> String {
    let mut normalized = body.trim().to_owned();
    if !normalized.ends_with(['.', '!', '?']) {
        normalized.push('.');
    }
    normalized
}

fn agent_name() -> Result<agent::Name> {
    agent::Name::try_new("daily-care-update").map_err(invalid_domain_value)
}

fn audit_action_label(label: &str) -> Result<entities::audit::ActionLabel> {
    entities::audit::ActionLabel::try_new(label).map_err(invalid_domain_value)
}

fn flag_message(message: &str) -> Result<FlagMessage> {
    FlagMessage::try_new(message).map_err(invalid_domain_value)
}

fn invalid_domain_value(error: impl std::fmt::Display) -> Error {
    Error::InvalidDomainValue(error.to_string())
}
