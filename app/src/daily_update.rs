use bon::Builder;
use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::agents;
use crate::agents::WorkflowAgent;
use domain::{agent, audit, customer, entities, message, pet, policy, workflow};

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("daily update preview requires a DailyNoteCreated or DailyUpdateNeeded workflow event")]
    UnsupportedWorkflowEvent,
    #[error("daily update preview requires at least one staff note")]
    MissingStaffNotes,
    #[error("daily update preview requires at least one policy-allowed draft/summarize action")]
    MissingAllowedAction,
    #[error("daily update preview could not build a validated domain value: {0}")]
    InvalidDomainValue(String),
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct MvpPreviewRequest {
    pub event: workflow::Event,
    pub pet_name: pet::Name,
    pub owner_display_name: customer::Name,
    pub policy_snapshot_id: policy::Id,
    pub notes: Vec<entities::CareNote>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MvpPreview {
    pub agent_packet: agents::AgentPromptPacket<DailyCareUpdateInput>,
    pub output: DailyCareUpdateOutput,
    pub owner_message_draft: CustomerMessageDraft,
    pub approval: entities::ApprovalRecord,
    pub send_stub: SendStub,
    pub audit_log: Vec<entities::AuditEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyCareUpdateInput {
    pub pet_name: pet::Name,
    pub owner_display_name: customer::Name,
    pub policy_snapshot_id: policy::Id,
    pub notes: Vec<entities::CareNote>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyCareUpdateOutput {
    pub customer_message: CustomerMessageDraft,
    pub internal_flags: Vec<InternalFlag>,
    pub should_send: bool,
    pub requires_review: bool,
    pub review_reason: Option<ReviewReason>,
    pub included_facts: Vec<IncludedFact>,
    pub omitted_facts: Vec<OmittedFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomerMessageDraft {
    pub body_ref: message::BodyRef,
    pub channel_hint: message::Channel,
    pub language: LanguageTag,
    pub tone: ToneLabel,
    pub audience: Audience,
    pub redaction_profile: RedactionProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Audience {
    Customer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InternalFlagCode {
    CustomerMessageApprovalNotConfigured,
    RawInternalNoteNotCustomerSafe,
    BehaviorReviewRequired,
    MedicalOrMedicationReviewRequired,
    PolicyGapRequiresReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InternalFlagSeverity {
    Info,
    NeedsStaffReview,
    NeedsManagerReview,
    DoNotSend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendedFlagAction {
    StaffReview,
    ManagerReview,
    SuppressUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalFlag {
    pub code: InternalFlagCode,
    pub severity: InternalFlagSeverity,
    pub message: FlagMessage,
    pub source_note_ids: Vec<entities::CareNoteId>,
    pub recommended_action: RecommendedFlagAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncludedFact {
    pub source_note_id: entities::CareNoteId,
    pub summary: FactSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmittedFact {
    pub source_note_id: entities::CareNoteId,
    pub reason: OmissionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OmissionReason {
    InternalOnly,
    SensitiveRequiresReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendStub {
    pub mode: SendMode,
    pub blocked_by: Vec<policy::ReviewGate>,
    pub audit_action: entities::AuditAction,
}

impl SendStub {
    pub fn is_blocked_until_human_approval(&self) -> bool {
        matches!(self.mode, SendMode::ApprovalRequiredStub) && !self.blocked_by.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendMode {
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

pub fn build_mvp_preview(request: MvpPreviewRequest) -> Result<MvpPreview> {
    validate_request(&request)?;

    let input = DailyCareUpdateInput {
        pet_name: request.pet_name,
        owner_display_name: request.owner_display_name,
        policy_snapshot_id: request.policy_snapshot_id,
        notes: request.notes,
    };

    let agent = DailyCareUpdateAgent;
    let agent_packet = agent.build_prompt_packet(&request.event, input.clone());
    let output = generate_output(&input)?;
    let review_gate = review_gate_for(&output);

    let message_id =
        entities::MessageId(Uuid::from_u128(0xDA17_0000_0000_0000_0000_0000_0000_0001));
    let approval_id =
        entities::ApprovalId(Uuid::from_u128(0xDA17_0000_0000_0000_0000_0000_0000_0002));
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

    let approval = entities::ApprovalRecord::builder()
        .id(approval_id)
        .target(entities::ApprovalTarget::Message(message_id))
        .gate(review_gate.clone())
        .lifecycle(entities::ApprovalLifecycle::ApprovalRequested)
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
        audit_action: entities::AuditAction::Extension(audit_action_label(
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

#[derive(Debug, Clone, Copy)]
pub struct DailyCareUpdateAgent;

impl agents::WorkflowAgent<DailyCareUpdateInput, DailyCareUpdateOutput> for DailyCareUpdateAgent {
    fn spec(&self) -> agents::AgentSpec {
        agents::baseline_agent_specs()
            .into_iter()
            .find(|spec| spec.name.clone().into_inner() == "daily-care-update")
            .expect("baseline daily-care-update agent spec exists")
    }

    fn build_prompt_packet(
        &self,
        event: &workflow::Event,
        input: DailyCareUpdateInput,
    ) -> agents::AgentPromptPacket<DailyCareUpdateInput> {
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
        mut output: workflow::Result<DailyCareUpdateOutput>,
    ) -> workflow::Result<DailyCareUpdateOutput> {
        if let Some(structured_output) = output.structured_output.as_mut() {
            structured_output.should_send = false;
            structured_output.requires_review = true;
        }
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

fn generate_output(input: &DailyCareUpdateInput) -> Result<DailyCareUpdateOutput> {
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
        if matches!(note.visibility, entities::CareNoteVisibility::InternalOnly) {
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
            .any(|note| matches!(note.kind, entities::CareNoteKind::Behavior))
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

    Ok(DailyCareUpdateOutput {
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
        should_send: false,
        requires_review: true,
        review_reason: Some(ReviewReason::try_new(review_reason).map_err(invalid_domain_value)?),
        included_facts,
        omitted_facts,
    })
}

fn review_gate_for(output: &DailyCareUpdateOutput) -> policy::ReviewGate {
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
    approval_id: entities::ApprovalId,
) -> Result<Vec<entities::AuditEvent>> {
    Ok(vec![
        audit_event(
            event.occurred_at,
            event.actor.clone(),
            entities::AuditSubject::WorkflowEvent(event.event_id),
            entities::AuditAction::WorkflowEventRecorded,
            "daily-care-update workflow event recorded for MVP preview",
        )?,
        audit_event(
            event.occurred_at,
            entities::ActorRef::Agent {
                workflow: agent_name()?,
            },
            entities::AuditSubject::Message(message_id),
            entities::AuditAction::MessageApprovalRequested,
            "daily care update owner-message draft created; no live send attempted",
        )?,
        audit_event(
            event.occurred_at,
            entities::ActorRef::Agent {
                workflow: agent_name()?,
            },
            entities::AuditSubject::Approval(approval_id),
            entities::AuditAction::ApprovalDecisionRecorded,
            "approval record opened for staff/manager review stub",
        )?,
    ])
}

fn audit_event(
    at: DateTime<Utc>,
    actor: entities::ActorRef,
    subject: entities::AuditSubject,
    action: entities::AuditAction,
    summary: &str,
) -> Result<entities::AuditEvent> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        entities::AuditMetadataKey::try_new("summary").map_err(invalid_domain_value)?,
        entities::AuditMetadataValue::try_new(summary).map_err(invalid_domain_value)?,
    );
    Ok(entities::AuditEvent {
        at,
        actor,
        subject,
        action,
        metadata,
    })
}

fn subject_reservation_id(event: &workflow::Event) -> entities::ReservationId {
    match event.subject {
        workflow::Subject::Reservation(id) => id,
        _ => entities::ReservationId(Uuid::nil()),
    }
}

fn sensitive_kind(kind: entities::CareNoteKind) -> bool {
    matches!(
        kind,
        entities::CareNoteKind::Medication
            | entities::CareNoteKind::Medical
            | entities::CareNoteKind::Behavior
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

fn audit_action_label(label: &str) -> Result<entities::AuditActionLabel> {
    entities::AuditActionLabel::try_new(label).map_err(invalid_domain_value)
}

fn flag_message(message: &str) -> Result<FlagMessage> {
    FlagMessage::try_new(message).map_err(invalid_domain_value)
}

fn invalid_domain_value(error: impl std::fmt::Display) -> Error {
    Error::InvalidDomainValue(error.to_string())
}
