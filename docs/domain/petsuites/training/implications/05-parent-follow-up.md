# Training implication 05: Parent follow-up

Purpose: model the operational and domain contract for Training parent follow-up in the PetSuites/NVA AI-operations foundation. This is a modeling artifact for later Rust/domain cards, not an implementation patch. It uses the Training service map as the parent contract and keeps member-facing communication behind explicit review gates.

Assumption: "parent" means the customer/owner contact for the pet. The safest extensible model treats follow-up as a training workflow outcome that may create internal staff tasks and draft messages, but never sends a customer/member-facing message or asserts training outcomes without trainer/staff approval.

## 1. Operational story

### Trigger

Parent follow-up is triggered when one of these Training facts becomes true:

- A training session is completed and the active `operations::training::Contract` has `FollowUpCadence::AfterEachSession`.
- A multi-session or Stay and Study program reaches completion and the contract has `FollowUpCadence::AfterProgramCompletion`.
- A completed program reaches a later cadence checkpoint such as `FollowUpCadence::ThirtyDaysAfterCompletion`.
- Trainer notes, progress milestones, homework, or outcome documentation are missing or stale before a due parent update.
- A parent asks for training progress, homework clarification, next steps, rebooking, or package continuation.
- A policy detects a safe re-enrollment/package opportunity after completion, unused sessions, lapsed practice, or unfinished curriculum milestones.
- A behavior, care, incident, complaint, or ambiguous outcome signal appears in the follow-up context and requires escalation instead of ordinary parent coaching.

### Actors

- Parent/customer: receives approved updates, homework, rebooking prompts, and escalation responses.
- Pet: the subject of training progress, milestones, outcomes, and homework.
- Trainer: owns training evidence, progress interpretation, homework instructions, and ordinary parent coaching approval.
- Front desk or lead staff: may queue and send approved operational messages, schedule follow-up calls, and coordinate rebooking prompts.
- Manager: approves escalations, complaints, safety-sensitive behavior language, capacity overrides, refunds/credits, and exception handling.
- AI workflow agent: drafts summaries, detects missing evidence, recommends staff tasks, and prepares safe message drafts; it does not approve or send.
- Workflow/repository layer: persists follow-up state, audit records, draft messages, staff tasks, and event links.

### Inputs

- `operations::training::EnrollmentId`, customer/pet/location IDs, and optional reservation ID.
- Enrollment status, program kind, package policy, and follow-up cadence from `operations::training::Enrollment`/`Contract`.
- Trainer assignment and trainer requirement/qualification.
- Session completion or program completion event with timestamp and staff actor.
- `operations::training::ProgressReport`, `ProgressEvidence`, `CurriculumMilestone`, and `MilestoneStatus` values.
- `operations::training::OutcomeDocumentation` when an outcome or graduation claim is present.
- Existing customer contact preference from customer/entities, not raw contact fields embedded in Training.
- Care profile, temperament, incident, and reservation context needed for safety review.
- Location-level follow-up rules, message templates, allowed channels, quiet hours, and escalation thresholds.
- Payment/package state when the follow-up includes package usage, renewal, credit, or re-enrollment.

### Decisions

- Is a follow-up due now, already satisfied, deferred, or blocked by missing evidence?
- Is the follow-up session-level, program-completion, 30-day check-in, parent-requested, escalation, or re-enrollment oriented?
- Can the workflow produce only an internal task, a trainer review task, a manager review task, or a customer message draft?
- Does the follow-up need trainer approval, staff message approval, manager approval, or complete suppression from automation?
- Does the progress evidence support the proposed homework/instructions/outcome wording?
- Is any behavior/care/incident language safety-sensitive enough to require manager review before parent contact?
- Should the next action be a message, call task, rebooking prompt, package review, trainer handoff, or manager escalation?
- What audit evidence links the generated draft/task to source notes, approvals, and send boundaries?

### Outputs

- `operations::training::ParentFollowUp` aggregate in a draft, pending-review, approved, sent, deferred, or escalated state.
- `operations::training::FollowUpPlan` describing due time, channel, audience, purpose, required evidence, and review gates.
- `operations::StaffTask` using either existing `CustomerFollowUp` or a future training-specific kind such as `TrainingParentFollowUp`, `TrainingHomeworkReview`, or `TrainingOutcomeApproval`.
- `workflow::RecommendedAction::InternalTask` for staff work and `workflow::RecommendedAction::DraftMessage` for unsent customer communication.
- `workflow::RecommendedAction::RequestHumanReview(policy::ReviewGate::CustomerMessageApproval)` or manager/training-specific review gate when needed.
- Audit event linking enrollment, session/progress/outcome records, source agent packet, reviewer, approval decision, and provider send reference if later sent.
- Optional safe revenue recommendation such as re-enrollment or package continuation, represented as staff review work rather than direct sales outreach.

### Success state

A successful parent follow-up has all of these properties:

- The due follow-up has a typed purpose and source event.
- Required trainer evidence is attached or the workflow is explicitly a missing-evidence task.
- Member-facing language has been reviewed and approved by the truthful human owner before send.
- Homework and next steps are specific enough to be useful, but do not invent diagnoses, guarantees, or unsupported outcome claims.
- The staff task or draft message is assigned to the right role and has a clear due time.
- The audit trail records who/what generated the draft, which source evidence was used, which gates were required, who approved, and what was sent or deferred.
- Follow-up completion can be queried without inspecting raw message bodies or provider logs.

### Failure and exception states

- Missing trainer evidence: create a trainer task; do not draft confident parent progress language.
- Missing customer contact preference or forbidden channel: create an internal task to resolve contact preference; do not choose a channel by guessing.
- Behavior/care/incident ambiguity: route to manager review and suppress normal coaching text.
- Unapproved member-facing draft: remain `PendingApproval`; no provider-send action may be emitted.
- Outcome claim without documentation: request outcome documentation approval before graduation/CGC/behavior-improvement wording appears.
- Package/payment ambiguity: route to staff/manager review; do not promise pricing, credits, refunds, or package availability.
- Trainer unavailable for requested follow-up call: create scheduling/reassignment task; do not confirm a call time.
- Duplicate follow-up for the same cadence window: collapse into one open follow-up or mark the newer request as already-covered with audit linkage.
- Parent complaint or negative sentiment: route to manager/lead staff, preserve facts, and avoid AI-authored defensive language.
- Provider/tool send failure: keep the approved draft and audit attempt; mark delivery failed and create retry/manual-send task.

## 2. Domain types to add or refine

Use `operations::training` as the semantic owner for training-specific follow-up. Keep contact identity, provider-send mechanics, payment capture, and general workflow execution in their existing modules.

### Core aggregate and identities

- `operations::training::ParentFollowUpId`
  - Stable ID for a follow-up record. It must not be interchangeable with reservation, workflow, or message IDs.
- `operations::training::ParentFollowUp`
  - Aggregate that links enrollment, pet, customer, location, trigger, purpose, state, plan, review boundary, draft/action outputs, and audit refs.
  - Invariant: cannot become `ApprovedForSend` or `Sent` without approval evidence for every required review gate.
- `operations::training::FollowUpPlan`
  - Due date/time, allowed channel, intended audience, purpose, required source evidence, assigned owner role, and review gates.
  - Invariant: a plan with customer-facing intent must have a known contact preference/channel or be blocked.
- `operations::training::FollowUpAuditTrail`
  - Generation source, source evidence refs, agent spec/output refs, reviewer approvals, send attempt refs, and final disposition.
  - Invariant: every state transition after draft creation records actor and timestamp.

### Trigger, purpose, and state enums

- `operations::training::FollowUpTrigger`
  - `SessionCompleted { session_id: TrainingSessionId }`
  - `ProgramCompleted { enrollment_id: EnrollmentId }`
  - `CadenceDue { cadence: FollowUpCadence }`
  - `ParentRequested { source_event_id: workflow::WorkflowEventId }`
  - `MissingProgressEvidence`
  - `ReEnrollmentOpportunity`
  - `EscalationRequired { reason: FollowUpEscalationReason }`

- `operations::training::FollowUpPurpose`
  - `ProgressUpdate`
  - `HomeworkCoaching`
  - `ProgramCompletionSummary`
  - `ThirtyDayCheckIn`
  - `ReEnrollmentOrPackageReview`
  - `ScheduleTrainerCall`
  - `MissingEvidenceResolution`
  - `ComplaintOrConcernEscalation`

- `operations::training::FollowUpState`
  - `NotDue`
  - `Due`
  - `Drafted`
  - `PendingTrainerApproval`
  - `PendingCustomerMessageApproval`
  - `PendingManagerApproval`
  - `ApprovedForSend`
  - `Sent`
  - `Deferred { reason: FollowUpDeferralReason }`
  - `Blocked { reason: FollowUpBlocker }`
  - `Escalated { reason: FollowUpEscalationReason }`
  - `Cancelled`

- `operations::training::FollowUpBlocker`
  - `MissingTrainerNotes`
  - `MissingOutcomeDocumentation`
  - `MissingCustomerContactPreference`
  - `UnsafeBehaviorLanguage`
  - `CareOrMedicalReviewRequired`
  - `IncidentOrComplaintRequiresManager`
  - `PackageOrPaymentReviewRequired`
  - `DuplicateOpenFollowUp`

- `operations::training::FollowUpEscalationReason`
  - `BehaviorSafetyConcern`
  - `IncidentRelated`
  - `ParentComplaint`
  - `OutcomeDisputedOrAmbiguous`
  - `TrainerUnavailableForRequestedCall`
  - `RefundCreditOrPackageException`

- `operations::training::FollowUpDeferralReason`
  - `WaitingForNextSession`
  - `ParentRequestedLaterContact`
  - `TrainerNeedsMoreObservation`
  - `LocationQuietHours`
  - `ManualStaffDecision`

### Message, homework, and approval values

- `operations::training::HomeworkInstruction`
  - Non-empty bounded parent-practice instruction from the service map. Construction proves text shape only; it does not prove approval for send.
- `operations::training::ParentCoachingSummary`
  - Non-empty bounded staff/trainer-authored summary for parent-facing coaching.
- `operations::training::FollowUpDraft`
  - Draft body, channel, purpose, evidence refs, generated_by, and required approvals.
  - Invariant: draft cannot carry `approved_for_send = true` as a boolean. Approval is represented by typed `FollowUpApproval` values.
- `operations::training::FollowUpApproval`
  - `TrainerApproved { staff_id, approved_at, evidence_refs }`
  - `CustomerMessageApproved { staff_id, approved_at }`
  - `ManagerApproved { staff_id, approved_at, reason }`
  - Invariant: approver roles must satisfy the required gate; AI actors cannot construct final approval.
- `operations::training::FollowUpDelivery`
  - `NotRequested`, `ReadyForProviderSend`, `Sent { provider, external_id, sent_at }`, `Failed { provider, reason }`.
  - Invariant: `ReadyForProviderSend` requires `ApprovedForSend` state.

### Policy and contract refinements

- Refine `operations::training::FollowUpCadence` from contract-level configuration into an input to `FollowUpPolicy`, not the whole follow-up concept.
- Add `operations::training::FollowUpReviewPolicy` or fold review decisions into `FollowUpPolicy` if the review decision is inseparable from due/draft planning.
- Add training-specific review gate vocabulary before behavior depends on raw `policy::ReviewGate`. Until then map safely to existing `policy::ReviewGate::{CustomerMessageApproval, ManagerApproval, BehaviorReview}`.
- Add training-specific staff task kinds before stuffing operational meaning into `CustomerFollowUp.reason` strings.
- Add `workflow::WorkflowEventType` variants such as `TrainingSessionCompleted`, `TrainingFollowUpDue`, and `TrainingOutcomeDocumentationNeeded` when serialized events are implemented.

## 3. Relationship map between types

### Entities and identity values

- `entities::CustomerId` identifies the parent/customer. Training references it; customer/contact modules own contact details and preferences.
- `entities::PetId` identifies the pet. Training references it; pet/care/temperament modules own health and behavior facts.
- `entities::LocationId` scopes configured offerings, allowed channels, staffing, templates, and cadence policy.
- `entities::ReservationId` is optional and present when the follow-up came from boarding/daycare-attached tutor sessions or Stay and Study.
- `entities::StaffId` identifies trainer, reviewer, sender, and manager actors.

### Training entities and value objects

- `operations::training::Enrollment` owns enrollment lifecycle and references the active program/package/follow-up contract.
- `operations::training::ProgressReport` and `ProgressEvidence` supply facts for the follow-up draft.
- `operations::training::OutcomeDocumentation` supplies approved outcome claims for completion/graduation language.
- `operations::training::ParentFollowUp` coordinates plan, draft, approval, staff tasks, and audit for one due follow-up.
- `operations::training::FollowUpPlan` determines due behavior and truthful task/message targets.
- `operations::training::FollowUpDraft` contains not-yet-approved member-facing text.
- `operations::training::FollowUpApproval` transitions drafts through review gates without booleans.

### Policies

- `operations::training::FollowUpPolicy` owns whether a follow-up is due and what plan should be created.
- `operations::training::FollowUpDraftPolicy` owns whether evidence is sufficient to draft a progress/homework/completion message.
- `operations::training::FollowUpReviewPolicy` owns review gate selection for customer-facing drafts and escalation cases.
- `operations::training::RecurringEngagementPolicy` owns safe re-enrollment/package recommendations and feeds follow-up purpose; it does not send sales messages.
- `operations::training::ProgressReportPolicy` remains the owner of progress report safety; follow-up policy consumes its decision rather than duplicating it.

### Repositories and stores

- `operations::training::follow_up::Repository`
  - Loads open/due follow-ups by enrollment, customer, pet, location, cadence window, and state.
  - Saves state transitions and audit links.
- `operations::training::progress::Repository`
  - Provides progress reports, evidence, milestones, and outcome docs used by follow-up.
- `operations::training::enrollment::Repository`
  - Provides enrollment/program/package/status context.
- `operations::training::trainer::ScheduleRepository`
  - Provides trainer availability for call/review tasks.
- `workflow::Repository` or workflow event store
  - Appends workflow events and recommended actions once domain policies produce them.

### Workflow events, staff tasks, and agent specs/tools

- `workflow::WorkflowEvent` should represent follow-up due/session completion/parent request events using typed subjects and policy context.
- `workflow::RecommendedAction::InternalTask` should carry trainer/manager/front-desk work without mutating provider records.
- `workflow::RecommendedAction::DraftMessage` should carry only unapproved drafts unless approval evidence is already present in domain state.
- `operations::StaffTask` should add training-specific kinds or typed reasons:
  - `TrainingParentFollowUp { follow_up_id }`
  - `TrainingHomeworkReview { enrollment_id }`
  - `TrainingOutcomeApproval { enrollment_id }`
  - `TrainingParentConcernEscalation { follow_up_id }`
- `agents::AgentSpec` should add a training follow-up assistant spec only after deterministic policy contracts exist. Its allowed tools should be read/draft/task-create only; forbidden actions should include sending, approving outcomes, diagnosing behavior, promising availability, or changing packages.

## 4. Interaction contract

Rust-like pseudo-signatures use semantic owners; avoid free-floating helper functions.

```rust
impl operations::training::ParentFollowUp {
    pub fn plan_due(
        enrollment: &operations::training::Enrollment,
        trigger: operations::training::FollowUpTrigger,
        evidence: operations::training::FollowUpEvidenceSnapshot,
        policy: &operations::training::FollowUpPolicy,
    ) -> operations::training::Result<operations::training::ParentFollowUp>;

    pub fn attach_draft(
        self,
        draft: operations::training::FollowUpDraft,
    ) -> operations::training::Result<Self>;

    pub fn record_approval(
        self,
        approval: operations::training::FollowUpApproval,
    ) -> operations::training::Result<Self>;

    pub fn mark_sent(
        self,
        delivery: operations::training::FollowUpDelivery,
    ) -> operations::training::Result<Self>;

    pub fn required_reviews(&self) -> &[operations::training::TrainingReviewGate];
    pub fn is_member_facing(&self) -> bool;
    pub fn is_ready_for_provider_send(&self) -> bool;
}
```

Behavior ownership:

- `ParentFollowUp` owns state transitions because it can verify review gates and audit invariants.
- `FollowUpPolicy` owns due/planning decisions because due behavior depends on cadence, trigger, evidence, and location rules.
- `FollowUpDraftPolicy` owns draftability and unsafe-language decisions because it interprets progress evidence and member-facing risk.
- `FollowUpService` orchestrates repositories/policies and emits workflow outputs; it should not hide policy decisions in generic helpers.

```rust
pub trait operations::training::FollowUpPolicy {
    fn plan(
        &self,
        enrollment: &operations::training::Enrollment,
        trigger: operations::training::FollowUpTrigger,
        evidence: &operations::training::FollowUpEvidenceSnapshot,
        location_rules: &operations::training::LocationFollowUpRules,
    ) -> operations::training::FollowUpDecision;
}

pub enum operations::training::FollowUpDecision {
    NotDue,
    CreatePlan(operations::training::FollowUpPlan),
    CreateMissingEvidenceTask(operations::training::MissingEvidenceTaskSpec),
    Escalate(operations::training::FollowUpEscalation),
    Suppress(operations::training::FollowUpBlocker),
}
```

```rust
pub trait operations::training::FollowUpDraftPolicy {
    fn draftability(
        &self,
        plan: &operations::training::FollowUpPlan,
        evidence: &operations::training::FollowUpEvidenceSnapshot,
        progress_decision: &operations::training::ProgressReportDecision,
    ) -> operations::training::FollowUpDraftDecision;
}

pub enum operations::training::FollowUpDraftDecision {
    DraftAllowed {
        required_reviews: Vec<operations::training::TrainingReviewGate>,
    },
    InternalTaskOnly {
        task: operations::training::StaffTaskSpec,
    },
    ManagerReviewRequired {
        reason: operations::training::FollowUpEscalationReason,
    },
    Blocked {
        reason: operations::training::FollowUpBlocker,
    },
}
```

```rust
pub trait operations::training::follow_up::Repository {
    fn load(
        &self,
        id: operations::training::ParentFollowUpId,
    ) -> operations::training::Result<Option<operations::training::ParentFollowUp>>;

    fn open_for_enrollment(
        &self,
        enrollment_id: operations::training::EnrollmentId,
    ) -> operations::training::Result<Vec<operations::training::ParentFollowUp>>;

    fn due_before(
        &self,
        location_id: entities::LocationId,
        cutoff: chrono::DateTime<chrono::Utc>,
    ) -> operations::training::Result<Vec<operations::training::ParentFollowUp>>;

    fn save_transition(
        &self,
        follow_up: &operations::training::ParentFollowUp,
        audit: operations::training::FollowUpAuditEntry,
    ) -> operations::training::Result<()>;
}
```

```rust
impl operations::training::FollowUpService {
    pub fn handle_trigger(
        &self,
        trigger: operations::training::FollowUpTrigger,
    ) -> operations::training::Result<operations::training::FollowUpWorkflowPlan>;

    pub fn produce_recommended_actions(
        &self,
        follow_up: &operations::training::ParentFollowUp,
    ) -> operations::training::Result<Vec<workflow::RecommendedAction>>;

    pub fn approve_for_send(
        &self,
        follow_up_id: operations::training::ParentFollowUpId,
        approval: operations::training::FollowUpApproval,
    ) -> operations::training::Result<operations::training::ParentFollowUp>;
}
```

Service behavior:

- `handle_trigger` loads enrollment, evidence, existing open follow-ups, customer preference, and location rules; it creates at most one active follow-up for the same enrollment/cadence/purpose window.
- `produce_recommended_actions` may emit internal tasks and draft messages, but never provider-send commands.
- `approve_for_send` records human approval and allows a later boundary adapter/tool card to perform provider send.
- Provider send belongs to a tool/boundary module that accepts `ParentFollowUp` only when `is_ready_for_provider_send()` is true.

## 5. Review and approval contract

### Automation level

- AI/deterministic automation may classify triggers, summarize trainer evidence, identify missing documentation, draft internal task text, draft customer message text, recommend re-enrollment/package review, and prepare audit packets.
- AI/deterministic automation may not approve, send, diagnose behavior, claim outcomes, guarantee improvement, promise availability, change schedules, alter packages, apply discounts, or handle refunds/credits.
- Deterministic policies, not model confidence, choose `AutomationLevel` and required review gates.

### Review gates

- Trainer review required:
  - Homework/coaching instructions.
  - Progress interpretation from trainer notes.
  - Milestone completion language.
  - Ordinary session/program summary.
- Customer message approval required:
  - Any customer/member-facing message draft.
  - Any rebooking/package prompt sent to the parent.
- Manager approval required:
  - Complaints, incidents, disputed outcomes, safety-sensitive behavior language.
  - Package/payment/refund/credit exceptions.
  - Overrides of trainer availability, waitlist, or schedule constraints.
  - Ambiguous CGC/graduation/readiness claims.
- Behavior/care review required:
  - Reactivity, bite/aggression, medical/medication, or special-handling facts that change safe wording or follow-up action.

Until training-specific review gates exist, map the above to existing gates:

- `policy::ReviewGate::CustomerMessageApproval` for outbound parent messages.
- `policy::ReviewGate::ManagerApproval` for escalation/exception handling.
- `policy::ReviewGate::BehaviorReview` for behavior-sensitive follow-up.
- Add training-specific gates later as `operations::training::TrainingReviewGate` and compose them into `policy` at the workflow boundary.

### Audit trail

Every parent follow-up must persist:

- Trigger and source event IDs.
- Enrollment/customer/pet/location/reservation IDs.
- Evidence refs used to draft or block the follow-up.
- Agent spec/name and model/tool output reference when AI contributed.
- Policy decisions and required gates.
- Human reviewers, roles, approval timestamps, and rejection/deferral reasons.
- Draft body/version hash or provider draft reference.
- Send attempt provider/reference/time/result if a later tool sends it.

### Customer/member-facing boundaries

- Customer-facing text is a draft until approved by the appropriate human gate.
- No direct provider-send command should be emitted from the Training domain or agent workflow.
- Staff-facing tasks can be created automatically when they do not change customer records or send messages.
- Revenue recommendations remain internal staff tasks/drafts until approved; no automated upsell send.
- Sensitive source notes should not leak through debug output or unreviewed message bodies.

## 6. Test contracts

Later code cards should write focused failing tests before implementation. Suggested tests:

### Domain contract tests

- `training_parent_follow_up_plan_is_due_after_each_session_when_contract_requires_session_cadence`
  - Given completed session evidence and `FollowUpCadence::AfterEachSession`, `FollowUpPolicy` creates a due progress/homework follow-up plan.
- `training_parent_follow_up_plan_is_due_after_program_completion_when_contract_requires_completion_cadence`
  - Program completion creates a completion-summary follow-up and does not create per-session duplicate work.
- `training_parent_follow_up_thirty_day_check_in_uses_completion_date_not_message_send_date`
  - The 30-day cadence is anchored to completion, with explicit deferral if a parent requested later contact.
- `parent_follow_up_draft_cannot_be_ready_for_send_without_customer_message_approval`
  - A draft message remains pending approval and `is_ready_for_provider_send()` is false.
- `parent_follow_up_records_trainer_approval_before_homework_instruction_can_be_sent`
  - Homework/coaching text requires trainer approval even if a customer-message reviewer is also present.
- `parent_follow_up_blocks_when_trainer_notes_are_missing`
  - Missing evidence produces a trainer task or blocker, not invented progress language.
- `parent_follow_up_escalates_behavior_sensitive_language_to_manager_or_behavior_review`
  - Reactivity/bite/incident facts choose review/escalation instead of ordinary update drafting.
- `parent_follow_up_collapses_duplicate_open_follow_up_for_same_enrollment_cadence_and_purpose`
  - Duplicate triggers do not create duplicate customer messages.
- `parent_follow_up_state_transition_requires_audit_actor_and_timestamp`
  - Draft/approval/send/defer/escalate transitions cannot happen without audit evidence.
- `parent_follow_up_outcome_claim_requires_outcome_documentation_approval`
  - CGC/graduation/outcome wording is blocked until outcome documentation carries review approval.

### Cross-module relationship tests

- `training_follow_up_references_customer_pet_location_and_optional_reservation_with_typed_ids`
  - The aggregate composes typed IDs and cannot swap reservation/customer/pet IDs.
- `training_follow_up_uses_customer_contact_preference_without_storing_raw_contact_fields`
  - Channel selection reads the customer/contact boundary rather than duplicating raw email/phone.
- `training_follow_up_creates_staff_task_assigned_to_trainer_for_missing_evidence`
  - Missing progress evidence creates a trainer-owned staff task.
- `training_follow_up_creates_manager_task_for_complaint_or_package_exception`
  - Complaint/refund/package exception routes to manager review.
- `training_follow_up_reenrollment_recommendation_is_internal_task_not_direct_customer_send`
  - Recurring engagement recommendations are staff tasks/drafts with customer-message approval gates.

### Workflow and agent boundary tests

- `training_follow_up_agent_can_draft_message_but_cannot_mark_message_approved`
  - Agent output can include `workflow::RecommendedAction::DraftMessage` but not approval evidence.
- `training_follow_up_agent_output_preserves_required_review_gates`
  - Validation rejects output that drops customer-message/manager/trainer review requirements.
- `training_follow_up_agent_cannot_emit_provider_send_or_schedule_mutation_action`
  - Tool/action validation rejects sends, booking confirmations, schedule changes, package updates, and refunds.
- `training_follow_up_workflow_result_failed_safely_when_policy_rejects_evidence`
  - Unsafe or incomplete evidence returns `NeedsHumanReview`/`RejectedByPolicy`, not `Completed` with a draft.

### Storage/serialization tests

- `training_parent_follow_up_roundtrips_state_plan_approvals_and_audit_refs`
  - Serialization preserves state, gates, approvals, source refs, and delivery state.
- `training_parent_follow_up_decode_rejects_ready_for_send_without_approval`
  - Storage conversion rejects impossible approved/send states.
- `training_follow_up_redacts_sensitive_source_notes_from_debug_output`
  - Debug/log rendering does not leak behavior/care note text.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add or split richer `operations::training` follow-up types, policies, error/result surface, staff task specs, and contract refinements.
- `domain/src/workflow.rs`
  - Add event/action variants for training follow-up due/completed/evidence-needed only after domain contracts stabilize.
- `domain/src/policy.rs`
  - Add or compose training-specific review gates if existing `ReviewGate` is too coarse.
- `domain/src/agents.rs`
  - Add a `training-parent-follow-up` agent spec with draft/task-only tools and explicit forbidden actions.
- `domain/tests/petsuites_training_parent_follow_up.rs`
  - Primary semantic domain tests for policies, state transitions, review gates, and duplicates.
- `domain/tests/training_agent_boundaries.rs`
  - Agent/workflow approval-boundary tests.
- `storage/tests/training_follow_up_storage.rs`
  - Serialization/roundtrip/rejection tests after storage rows exist.
- Storage crate/source files, once the serialized domain card is ready, for `follow_up` records and conversion boundaries.

### Migration/refactor risks

- `operations::training::FollowUpCadence` currently is only contract configuration. Do not overload it as the follow-up aggregate or state machine.
- Existing `operations::StaffTaskKind::CustomerFollowUp { reason }` may be too generic. Adding training-specific task kinds is safer than hiding Training meaning in raw reasons.
- Existing `policy::ReviewGate::CustomerMessageApproval` is broad. Training-specific gates may need to exist in `operations::training` and map into `policy` at workflow boundaries.
- Existing `workflow::RecommendedAction::DraftMessage` does not model approval evidence. Keep approval in Training domain state and let workflow carry review requests.
- Avoid making `agents` the source of policy truth. Agent specs validate boundaries; deterministic Training policies decide due/approval/unsafe states.
- Avoid raw message strings, booleans like `approved`, or status strings in storage rows; decode into semantic enums/newtypes immediately.
- Sensitive progress/behavior notes need redaction-safe debug/log behavior, especially if `ProgressNote` or `ParentCoachingSummary` are stored.
- Duplicate prevention needs a cadence/purpose window model before scheduled jobs poll due follow-ups.

### Dependencies on other implications and parent map

- Depends on Training service map recommendations for `Enrollment`, `ProgressReport`, `OutcomeDocumentation`, `TrainingReviewGate`, `ApprovalBoundary`, and repositories.
- Depends on progress reporting/outcome documentation modeling before safe completion summaries or graduation claims can be implemented.
- Depends on trainer availability/assignment modeling for trainer call scheduling and missing-evidence routing.
- Depends on recurring engagement/package policy for safe re-enrollment and package continuation prompts.
- Depends on workflow/agent approval-boundary contracts before any AI-generated draft reaches a customer-send adapter.

### Suggested implementation order

1. Add `ParentFollowUpId`, `FollowUpTrigger`, `FollowUpPurpose`, `FollowUpState`, blockers/escalation reasons, and review/approval types under `operations::training`.
2. Add `ParentFollowUp` state-transition methods with semantic errors and audit requirements.
3. Add `FollowUpPolicy` and `FollowUpDraftPolicy` tests for due/block/escalate decisions.
4. Add repository traits and service orchestration contracts without storage implementation.
5. Add workflow recommended-action conversion that emits internal tasks/drafts/review requests only.
6. Add agent spec validation after deterministic policy tests pass.
7. Add storage serialization after the state machine and impossible-state rejection tests are stable.
