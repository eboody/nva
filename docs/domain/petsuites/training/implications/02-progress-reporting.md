# Training implication 02: Progress reporting

Purpose: define the operational and domain contract for PetSuites training progress reporting before a later Rust implementation card serializes the model. This is a modeling artifact only. It assumes the Training service domain map in `docs/domain/petsuites/training/service-domain-map.md` is the parent contract, and it keeps AI behavior behind typed review gates.

Assumption used here: a progress report may be drafted after a single tutor/private lesson session, during a multi-session Stay and Study or group class program, or at program completion. The safest extensible model treats every report as evidence-backed, trainer-owned documentation first; parent-facing text is a separate approved projection of that report.

## 1. Operational story

### Trigger

Progress reporting begins when one of these events occurs:

- A trainer completes a scheduled training session.
- A curriculum milestone changes state during an active enrollment.
- A program reaches a configured reporting cadence, such as after each session, weekly during Stay and Study, or at program completion.
- A staff member or manager notices a stale/missing progress report.
- A parent asks for an update and the enrollment has trainer evidence that can be summarized.
- An agent detects a gap: missing trainer note, overdue milestone evidence, unresolved review gate, or completed program without outcome documentation.

The trigger should become a typed workflow event, not a raw string flag. Recommended event vocabulary:

- `operations::training::ProgressReportDue`
- `operations::training::ProgressEvidenceSubmitted`
- `operations::training::MilestoneStatusChanged`
- `operations::training::ProgressReportDrafted`
- `operations::training::ProgressReportApproved`
- `operations::training::ProgressReportRejected`
- `operations::training::ParentUpdateSent`

Those may later be wrapped by `workflow::WorkflowEventType` or stored as training-specific event payloads.

### Actors

- Trainer: owns session evidence, milestone status, skill observations, homework recommendations, and outcome claims.
- Front desk / resort staff: may create missing-documentation tasks, route questions, and prepare draft parent updates, but does not invent training evidence.
- Manager / lead trainer: approves safety-sensitive wording, ambiguous outcomes, CGC readiness claims, trainer-capacity exceptions, and customer escalations.
- Parent / customer: receives only approved parent-facing summaries, homework, next-step scheduling prompts, or escalation communication.
- AI agent: drafts summaries, detects missing fields, proposes staff tasks, and highlights risk. It never finalizes evidence, approval, or customer sends.
- Workflow/tool boundary: stores events, drafts messages/tasks, and may later write to a provider only after approval.

### Inputs

Required inputs for a report draft:

- `operations::training::EnrollmentId`
- `entities::CustomerId`
- `entities::PetId`
- `entities::LocationId`
- `operations::training::ProgramKind`
- `operations::training::SessionRef` or `operations::training::ProgramReportingWindow`
- `entities::StaffId` for the trainer/documenter
- One or more `operations::training::ProgressEvidence` values
- Current `operations::training::CurriculumMilestone` states when curriculum-based reporting is enabled
- `operations::training::ProgressTracking`/reporting cadence from the program contract
- Review/approval state: `operations::training::ApprovalBoundary` or a training-specific wrapper around `policy::ReviewGate`

Optional inputs:

- Reservation context for boarding/daycare-attached tutor sessions or Stay and Study.
- Care/temperament flags that should restrict wording or require review.
- Media references, if photo/video proof exists.
- Package/session usage and follow-up cadence, if the report should trigger re-enrollment or next-session tasks.
- Parent communication preference from the customer/contact modules.

### Decisions

The progress-reporting workflow answers these questions in order:

1. Is reporting required for this enrollment/program/session?
2. Does enough trainer evidence exist to create a report draft?
3. Which milestones can be updated, and which must remain unchanged or be deferred?
4. Does any evidence mention safety, reactivity, bite/aggression, medical/care restrictions, complaint context, or ambiguous outcome language?
5. Is the report internal-only, staff-reviewable parent draft, manager-reviewable parent draft, or unsafe for automation?
6. Does the report create follow-up work: homework, next session, consult, manager review, outcome documentation, or recurring engagement recommendation?
7. Is the customer/member-facing projection approved to send?

### Outputs

Primary outputs:

- `operations::training::ProgressReport` persisted in the training progress store.
- `operations::training::MilestoneProgressUpdate` values applied to the enrollment/curriculum state.
- `operations::training::ParentProgressUpdateDraft` for customer-facing communication, if safe to draft.
- `operations::training::ProgressReportApproval` or review requirement.
- Staff tasks for missing evidence, trainer review, manager approval, parent follow-up, or outcome documentation.
- Workflow events for auditability.

Secondary outputs:

- `operations::training::OutcomeDocumentationDraft` when progress implies completion/readiness.
- `operations::training::RecurringEngagementOpportunity` when follow-up/re-enrollment is appropriate.
- Agent prompt/output packets for summary generation and gap detection.

### Success state

A successful progress-reporting cycle has these truths:

- The report is tied to exactly one enrollment and a specific session or reporting window.
- Every reported milestone state is backed by trainer-authored evidence or explicit deferral reason.
- The internal report preserves the trainer's evidence separately from any parent-facing prose.
- Review gates are explicit and not implied by a boolean.
- Parent-facing output is either approved and sendable, or it remains a draft/task with a review gate.
- The audit trail records who/what drafted, who approved, when it was sent, and which evidence was used.
- No provider mutation or customer message happens from raw AI output alone.

### Failure and exception states

- Missing trainer evidence: create a trainer task; do not draft a substantive update from absence.
- Evidence present but not attributable to a trainer/staff actor: store as untrusted/imported evidence and require review.
- Enrollment not found or not in progress: reject with `operations::training::progress::Error::EnrollmentNotReportable`.
- Session/reporting window mismatch: reject with `operations::training::progress::Error::SessionNotInEnrollment`.
- Milestone regression or impossible transition: reject or route to review; do not silently overwrite prior state.
- Behavior/safety/medical terms detected: require trainer/manager review before parent-facing use.
- CGC readiness or outcome claim without required evidence: create outcome-documentation review task; do not mark ready.
- Parent-facing text generated but unapproved: persist only as draft with `CustomerMessageApproval`/training gate.
- Provider write/send fails: keep the approved draft and record a failed delivery event; do not downgrade approval into completion.

## 2. Domain types to add or refine

Keep detailed progress vocabulary under `operations::training` and, as behavior grows, under `operations::training::progress`. Use re-exports only to define one canonical public surface.

### IDs and refs

- `operations::training::ProgressReportId`: stable report identity; non-empty/UUID-backed and not swappable with enrollment/session IDs.
- `operations::training::SessionRef`: identifies a training session, class meeting, tutor session, or Stay and Study interval. It should encode whether the session is standalone or reservation-attached.
- `operations::training::ProgramReportingWindow`: `{ enrollment_id, starts_at, ends_at }`; invariant: `starts_at < ends_at` and window belongs to the enrollment.
- `operations::training::EvidenceId`: stable ID for a source note/media/scorecard item.
- `operations::training::ApprovalId`: stable approval decision ID for audit and provider-send traceability.

### Text/value objects

- `operations::training::ProgressNote`: non-empty bounded trainer note. Sensitive by default; do not display as parent-facing copy without transformation and approval.
- `operations::training::ParentFacingSummary`: non-empty bounded customer-safe summary. Construction means text shape is valid, not approved.
- `operations::training::TrainerObservation`: non-empty bounded internal observation; may include behavior details.
- `operations::training::HomeworkInstruction`: non-empty bounded parent coaching instruction; requires staff approval before send.
- `operations::training::DeferralReason`: non-empty bounded explanation for why a milestone was deferred.
- `operations::training::ScoreValue`: bounded score if scorecards become numeric; prefer a semantic enum if scoring is categorical.

### Progress aggregate

`operations::training::ProgressReport` should be a domain aggregate, not a formatted message.

Required fields:

- `report_id: ProgressReportId`
- `enrollment_id: EnrollmentId`
- `pet_id: entities::PetId`
- `location_id: entities::LocationId`
- `reporting_scope: ProgressReportingScope`
- `documented_by: entities::StaffId`
- `evidence: Vec<ProgressEvidence>`
- `milestone_updates: Vec<MilestoneProgressUpdate>`
- `approval: ProgressApprovalState`
- `created_at`

Invariants:

- Evidence cannot be empty.
- The documenting actor must be a trainer or authorized staff role for this enrollment.
- Parent-facing fields cannot be marked sendable unless approval is recorded.
- A report for a reporting window cannot claim a milestone outside the enrollment's curriculum.
- Outcome claims must reference evidence and a review gate.

### Enums

- `operations::training::ProgressReportingScope`
  - `Session { session_ref: SessionRef }`
  - `ProgramWindow { window: ProgramReportingWindow }`
  - `ProgramCompletion { enrollment_id: EnrollmentId }`
  - `ParentRequestedUpdate { request_event_id: workflow::WorkflowEventId }`

- `operations::training::EvidenceSource`
  - `TrainerEntered`
  - `ImportedProviderNote`
  - `StaffEnteredForTrainer`
  - `AgentExtractedDraft`
  - `MediaReference`

- `operations::training::ProgressEvidence`
  - `TrainerNote { evidence_id: EvidenceId, note: ProgressNote, source: EvidenceSource }`
  - `MilestoneObserved { evidence_id: EvidenceId, milestone_id: MilestoneId, status: MilestoneStatus }`
  - `Scorecard { evidence_id: EvidenceId, score: TrainerScore }`
  - `HomeworkRecommended { evidence_id: EvidenceId, instruction: HomeworkInstruction }`
  - `MediaAttached { evidence_id: EvidenceId, media_id: workflow::external::Id }`
  - `OutcomeCandidate { evidence_id: EvidenceId, outcome: Outcome }`

- `operations::training::MilestoneProgressUpdate`
  - `Introduce { milestone_id, evidence_id }`
  - `MarkPracticing { milestone_id, evidence_id }`
  - `MarkDemonstrated { milestone_id, evidence_id }`
  - `NeedsParentPractice { milestone_id, evidence_id, homework }`
  - `Defer { milestone_id, reason }`
  - `EscalateForReview { milestone_id, reason }`

- `operations::training::ProgressApprovalState`
  - `InternalOnly`
  - `DraftNeedsTrainerReview { gate: TrainingReviewGate }`
  - `DraftNeedsManagerReview { gate: TrainingReviewGate }`
  - `ApprovedForParentSend { approval: ProgressApproval }`
  - `RejectedForParentSend { reason: ProgressRejectionReason }`
  - `UnsafeForAutomation { reason: AutomationUnsafeReason }`

- `operations::training::ProgressRejectionReason`
  - `MissingEvidence`
  - `AmbiguousOutcomeClaim`
  - `UnsafeBehaviorLanguage`
  - `MedicalOrCareConcern`
  - `CustomerEscalation`
  - `IncorrectEnrollmentOrSession`

- `operations::training::TrainerScore`
  - `Introduced`
  - `Practicing`
  - `ConsistentWithTrainer`
  - `ConsistentWithParentPracticeNeeded`
  - `NeedsFollowUpPlan`

### Policies/services/stores to add

- `operations::training::progress::ReportPolicy`: owns evidence sufficiency, parent-facing safety, approval state, and milestone-transition rules.
- `operations::training::progress::Repository`: appends reports, loads latest report by enrollment, and prevents duplicate report windows.
- `operations::training::progress::AuditLog`: append-only approval/send/evidence trace if a generic workflow audit store is not enough.
- `operations::training::ProgressService`: orchestrates report creation from enrollment/curriculum/evidence/policy/stores.
- `operations::training::ParentUpdateService`: turns approved reports into draft/sendable workflow messages; it does not own training evidence.

## 3. Relationship map between types

### Entities

- `operations::training::Enrollment` is the root operational context. A progress report cannot exist without an enrollment.
- `operations::training::Program` and `operations::training::Curriculum` define which reporting cadence and milestones are meaningful.
- `operations::training::CurriculumMilestone` is updated by `MilestoneProgressUpdate`; it should not mutate itself from free-form notes.
- `entities::CustomerId`, `entities::PetId`, `entities::LocationId`, and optionally `entities::ReservationId` are referenced, not re-modeled.
- `entities::StaffId` identifies trainer/documenter/approver; `operations::StaffRole::Trainer` is role vocabulary, not enough by itself for qualification.

### Value objects

- `ProgressNote`, `TrainerObservation`, `HomeworkInstruction`, `ParentFacingSummary`, `DeferralReason`, `TrainerScore`, and IDs are semantically distinct. Avoid using raw `String`, raw `Uuid`, or `bool approved` in the domain core.
- `ProgramReportingWindow` and `SessionRef` prevent reports from floating outside the session/program context they claim to summarize.

### Policies

- `ProgressReportPolicy` decides whether evidence supports a report, which review gate applies, and whether parent-facing drafting is allowed.
- `CurriculumPolicy` decides which milestones exist and which transitions are valid for a program.
- `ReadinessPolicy` and `TrainerAvailabilityPolicy` feed context but should not own reporting behavior.
- `RecurringEngagementPolicy` may consume approved outcome/progress signals to propose follow-up, but it should not rewrite progress evidence.

### Repositories/stores

- `training::enrollment::Repository` loads enrollment/program/curriculum state.
- `training::progress::Repository` appends reports and queries latest/stale/missing reports.
- `training::curriculum::Repository` loads milestone templates/current curriculum versions.
- `workflow` or `training::progress::AuditLog` records draft, approval, send, reject, and provider-sync events.
- `workflow` draft stores hold `DraftTask` and `DraftMessage` outputs until review.

### Workflow events

- `ProgressReportDue` creates a trainer/staff task if no evidence exists.
- `ProgressEvidenceSubmitted` may cause the policy to draft a report.
- `ProgressReportDrafted` records the internal report and its approval state.
- `ProgressReportApproved` unlocks a parent update projection.
- `ParentUpdateSent` records external delivery after a provider/tool send succeeds.
- `ProgressReportRejected` keeps the audit trail and routes remediation work.

### Staff tasks

Add training-specific task variants instead of generic titles:

- `operations::StaffTaskKind::TrainingProgressReportDue { enrollment_id }`
- `operations::StaffTaskKind::TrainingProgressReview { report_id }`
- `operations::StaffTaskKind::TrainingParentUpdateApproval { report_id }`
- `operations::StaffTaskKind::TrainingOutcomeDocumentationReview { enrollment_id }`
- `operations::StaffTaskKind::TrainingHomeworkApproval { report_id }`

Assignments should use `StaffTaskAssignment::Staff(trainer_id)`, `Role(StaffRole::Trainer)`, or `Role(StaffRole::Manager)` according to the gate.

### Agent specs/tools

Add a baseline agent spec only after deterministic policy contracts exist:

- Agent name: `training-progress-reporter`
- Purpose: summarize trainer evidence, identify missing milestone/outcome fields, draft parent-safe update copy, and propose review tasks.
- Allowed tools: `training-enrollment-read`, `training-progress-read`, `curriculum-read`, `draft-message`, `task-create`.
- Forbidden actions: `invent evidence`, `mark milestone complete`, `claim CGC readiness`, `send customer message`, `change enrollment/session/provider record`.
- Default gates: `CustomerMessageApproval`, `BehaviorReview`, and `ManagerApproval` when outcome/safety/complaint language appears.

The agent output should be a typed draft packet, not a final report mutation.

## 4. Interaction contract

Rust-like pseudo-signatures below are contract shapes, not required exact APIs.

### Progress report construction

```rust
impl operations::training::ProgressReport {
    pub fn draft(
        enrollment: &operations::training::Enrollment,
        scope: operations::training::ProgressReportingScope,
        documented_by: entities::StaffId,
        evidence: Vec<operations::training::ProgressEvidence>,
        milestone_updates: Vec<operations::training::MilestoneProgressUpdate>,
        policy: &operations::training::progress::ReportPolicy,
    ) -> operations::training::progress::Result<Self>;

    pub fn approval_state(&self) -> &operations::training::ProgressApprovalState;
    pub fn evidence(&self) -> &[operations::training::ProgressEvidence];
    pub fn can_project_parent_update(&self) -> bool;
}
```

Ownership rule: the aggregate validates that a report is internally coherent. It does not call provider tools or send messages.

### Progress report policy

```rust
pub trait operations::training::progress::ReportPolicy {
    fn evaluate_evidence(
        &self,
        enrollment: &operations::training::Enrollment,
        scope: &operations::training::ProgressReportingScope,
        evidence: &[operations::training::ProgressEvidence],
    ) -> operations::training::progress::EvidenceDecision;

    fn decide_approval_boundary(
        &self,
        report: &operations::training::ProgressReport,
        care_context: Option<&entities::CareProfile>,
        temperament_context: Option<&temperament::Profile>,
    ) -> operations::training::ProgressApprovalState;

    fn validate_milestone_transition(
        &self,
        current: &operations::training::CurriculumMilestone,
        update: &operations::training::MilestoneProgressUpdate,
    ) -> operations::training::progress::Result<()>;
}
```

Ownership rule: the policy owns evidence sufficiency and review-gate selection. It does not persist reports.

### Progress service

```rust
impl operations::training::ProgressService {
    pub fn create_report_draft(
        &self,
        command: operations::training::CreateProgressReport,
    ) -> operations::training::progress::Result<operations::training::ProgressReportDrafted>;

    pub fn approve_parent_update(
        &self,
        report_id: operations::training::ProgressReportId,
        approval: operations::training::ProgressApproval,
    ) -> operations::training::progress::Result<operations::training::ParentProgressUpdateDraft>;

    pub fn reject_parent_update(
        &self,
        report_id: operations::training::ProgressReportId,
        reviewer: entities::StaffId,
        reason: operations::training::ProgressRejectionReason,
    ) -> operations::training::progress::Result<operations::training::ProgressReportRejected>;
}
```

Ownership rule: the service orchestrates repositories, policy, staff tasks, and workflow events. It still returns drafts/events rather than provider-side sends.

### Repository contracts

```rust
pub trait operations::training::progress::Repository {
    fn append(
        &mut self,
        report: operations::training::ProgressReport,
    ) -> operations::training::progress::Result<()>;

    fn get(
        &self,
        id: operations::training::ProgressReportId,
    ) -> operations::training::progress::Result<operations::training::ProgressReport>;

    fn latest_for_enrollment(
        &self,
        enrollment_id: operations::training::EnrollmentId,
    ) -> operations::training::progress::Result<Option<operations::training::ProgressReport>>;

    fn find_overdue(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> operations::training::progress::Result<Vec<operations::training::ProgressReportDue>>;
}
```

The progress repository should append immutable report records or versioned report records. Avoid an update-in-place model that can erase original trainer evidence.

### Parent update contract

```rust
impl operations::training::ParentUpdateService {
    pub fn draft_from_approved_report(
        &self,
        report: &operations::training::ProgressReport,
        customer_preferences: &customer::ContactPreference,
    ) -> operations::training::progress::Result<workflow::DraftMessage>;
}
```

Behavior rule: parent-facing message creation belongs to the parent-update service/projection. The training report remains the evidence-backed source of truth.

### Agent contract

```rust
pub struct operations::training::agent::ProgressReporter;

impl agents::WorkflowAgent<operations::training::ProgressReportPromptInput,
                           operations::training::ProgressReportAgentDraft>
    for operations::training::agent::ProgressReporter
{
    fn spec(&self) -> agents::AgentSpec;
    fn build_prompt_packet(...);
    fn validate_output(
        &self,
        output: workflow::WorkflowResult<operations::training::ProgressReportAgentDraft>,
    ) -> workflow::WorkflowResult<operations::training::ProgressReportAgentDraft>;
}
```

Validation must reject output that asserts evidence not present in the prompt input or marks a gate approved.

## 5. Review/approval contract

### Automation level

- Safe to automate:
  - Detect overdue/missing report candidates.
  - Create internal draft staff tasks.
  - Summarize evidence into an internal draft packet.
  - Classify obvious missing fields.
  - Prepare parent-message drafts that are explicitly not approved.

- Staff/trainer review required:
  - Any parent-facing training progress report.
  - Homework or parent coaching instructions.
  - Milestone completion/demonstration status.
  - Imported/provider notes that are not directly entered by the trainer.

- Manager/lead-trainer approval required:
  - CGC readiness or comparable outcome claims.
  - Behavior-sensitive, safety-sensitive, incident-adjacent, complaint-adjacent, or medical/care-adjacent wording.
  - Any report that changes an earlier outcome, contradicts prior evidence, or resolves an escalation.

- Never automate directly:
  - Inventing trainer evidence.
  - Sending customer messages without approval.
  - Marking an outcome achieved solely from AI summary.
  - Mutating provider/customer records from a report draft.
  - Downplaying aggression, bite, injury, reactivity, medical, or incident details.

### Gates

Training-specific gates should either wrap or map to existing `policy::ReviewGate` values:

- `TrainingReviewGate::TrainerProgressReview`
- `TrainingReviewGate::ParentMessageApproval` -> `policy::ReviewGate::CustomerMessageApproval`
- `TrainingReviewGate::BehaviorOrSafetyReview` -> `policy::ReviewGate::BehaviorReview`
- `TrainingReviewGate::OutcomeDocumentationApproval`
- `TrainingReviewGate::ManagerApproval` -> `policy::ReviewGate::ManagerApproval`

A report can satisfy multiple gates. Approval should be explicit per gate or represented by a composite approval that names the approved scope.

### Audit trail

Persist an audit record for:

- evidence creation/import/extraction source;
- report draft creation;
- policy decision and required gates;
- reviewer identity, role, decision, timestamp, and reason;
- parent-facing message draft creation;
- provider/tool send attempt and result;
- rejected or superseded reports.

`ProgressApproval` should include at least `{ approval_id, report_id, approved_by, approved_at, approved_scope, gates_satisfied }`. Approval scope matters: approving an internal summary is not the same as approving a customer send or CGC readiness claim.

### Customer/member-facing boundaries

- The internal report is not a customer message.
- Parent-facing copy is a projection from the report and must preserve the approved evidence boundaries.
- If approved copy omits sensitive details for tone, the internal report must still retain the full evidence and escalation path.
- AI-generated language must carry provenance as draft/agent-generated until reviewed.
- Sending through email/SMS/provider portal is a separate workflow/tool action after approval.

## 6. Test contracts

Later code cards should write focused failing tests before implementation. Suggested semantic test names:

### Domain construction tests

- `training_progress_report_requires_enrollment_scope_trainer_and_evidence`
  - Empty evidence, missing enrollment, or missing documenting trainer fails with semantic errors.

- `training_progress_report_session_scope_must_belong_to_enrollment`
  - A report cannot summarize a session from another enrollment/reservation.

- `training_progress_note_and_homework_instruction_reject_empty_or_overlong_text`
  - Text values enforce bounded non-empty invariants.

- `training_progress_report_preserves_internal_evidence_separately_from_parent_summary`
  - Parent-safe prose cannot replace the source trainer evidence.

### Milestone tests

- `training_milestone_update_requires_matching_curriculum_milestone`
  - Unknown milestone IDs are rejected.

- `training_milestone_cannot_move_to_demonstrated_without_trainer_evidence`
  - Demonstration status requires trainer evidence, not AI summary alone.

- `training_milestone_deferral_records_reason_without_erasing_prior_status`
  - Deferral preserves prior context and typed reason.

### Policy/review tests

- `training_progress_policy_requires_staff_approval_before_parent_facing_send`
  - Draft reports default to review unless explicit approval exists.

- `training_progress_policy_routes_behavior_or_safety_language_to_manager_review`
  - Behavior/safety terms trigger `BehaviorOrSafetyReview`/manager gate.

- `training_progress_policy_rejects_outcome_claim_without_outcome_documentation_gate`
  - CGC readiness/outcome claims cannot be final without review.

- `training_parent_update_approval_scope_does_not_approve_provider_send_by_accident`
  - Internal approval and send approval remain distinct.

### Repository/audit tests

- `training_progress_repository_appends_reports_without_mutating_prior_evidence`
  - Revisions create new versions/events or preserve original evidence.

- `training_progress_repository_finds_overdue_reports_from_reporting_cadence`
  - Missing report tasks arise from enrollment/program cadence.

- `training_progress_audit_records_drafter_reviewer_send_attempt_and_evidence_ids`
  - Audit trail remains complete across draft/approval/send lifecycle.

### Agent boundary tests

- `training_progress_agent_can_draft_parent_update_but_cannot_mark_report_approved`
  - Agent output remains draft/recommendation only.

- `training_progress_agent_output_validation_rejects_invented_milestones_or_evidence`
  - Output must reference prompt-provided evidence/milestone IDs.

- `training_progress_agent_routes_unsafe_language_to_review_instead_of_customer_send`
  - Unsafe classifications become tasks/gates, not messages.

### Cross-module tests

- `training_progress_report_links_customer_pet_location_and_optional_reservation_with_typed_ids`
  - IDs are not swappable raw UUIDs.

- `training_progress_follow_up_creates_staff_task_or_workflow_draft_with_review_gate`
  - Follow-up outputs are typed tasks/drafts with approval boundary.

- `training_progress_parent_update_uses_customer_contact_preference_without_storing_raw_contact_fields`
  - Communication projection composes customer module data instead of duplicating it.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Initial implementation may extend the existing inline `pub mod training` with progress types. If the module grows, split into `domain/src/operations/training/mod.rs`, `progress.rs`, `curriculum.rs`, `enrollment.rs`, and `error.rs` in a separate refactor card.

- `domain/src/workflow.rs`
  - Add workflow event payloads or subjects for training progress due/drafted/approved/sent/rejected.

- `domain/src/policy.rs`
  - Add or map review gates if training needs gate specificity beyond existing `ManagerApproval`, `BehaviorReview`, and `CustomerMessageApproval`.

- `domain/src/agents.rs`
  - Add `training-progress-reporter` baseline spec after deterministic policy tests exist.

- `domain/tests/petsuites_training_progress_reporting.rs`
  - New semantic domain tests for construction, policy, approval, and agent boundaries.

- `storage/tests/training_progress_storage.rs`
  - Later serialization/roundtrip tests for report/evidence/approval/audit persistence.

- Storage crate files, if present in the later card, for records/codecs that convert raw external/provider fields into semantic domain types.

### Migration/refactor risks

- Current `operations::training::ProgressTracking` is only a reporting strategy enum. Do not overload it as the report entity or approval state.
- Current `operations::training::Outcome` is a compact enum. Outcome documentation needs evidence/review state before it can safely drive parent-facing claims.
- Current `operations::training::Contract` has curriculum as `Vec<CurriculumUnit>` and outcomes as `Vec<Outcome>`. Progress reporting will need milestone IDs/statuses and reportable scopes; avoid stuffing these into raw vectors without identity.
- `operations::StaffTaskKind` currently lacks training-specific variants. Generic `CustomerFollowUp` or `DocumentReview` can bridge temporarily but should not carry the final training semantics.
- Existing `policy::ReviewGate` may be too coarse for training outcome approval. Add training-local gates and map outward rather than flattening all semantics into `CustomerMessageApproval`.
- If the later serialized Rust card keeps all operations code in one large `operations.rs`, the public paths can still be semantic, but the code may become hard to maintain. Prefer a module-split card once behavior exceeds simple contracts.
- Parent-facing serialization must not accidentally include internal/sensitive trainer notes through derived debug/display or broad DTO conversion.

### Dependencies on other implications

- Depends on enrollment/readiness modeling for `EnrollmentId`, reportable enrollment statuses, and optional reservation context.
- Depends on curriculum/milestone modeling for `CurriculumMilestone`, `MilestoneId`, and valid milestone transitions.
- Depends on trainer availability/assignment modeling for the authorized documenting trainer and staff review routing.
- Interacts with outcome documentation: progress can propose outcome candidates, but outcome approval should remain a distinct gate.
- Interacts with parent follow-up/recurring engagement: approved reports may generate homework, next-session prompts, or package/re-enrollment opportunities.
- Interacts with storage/serialization cards after domain invariants are implemented; storage should convert at the boundary and reject invalid report state.

### Suggested implementation slice

1. Add `ProgressReportId`, `EvidenceId`, `SessionRef`, report text newtypes, and `progress::Error`.
2. Add `ProgressEvidence`, `ProgressReportingScope`, `MilestoneProgressUpdate`, and `ProgressApprovalState` enums.
3. Add `ProgressReport` aggregate with smart constructor/builder and semantic tests.
4. Add `progress::ReportPolicy` with evidence sufficiency, milestone transition, and approval-boundary tests.
5. Add staff task/workflow event variants for missing report, review due, approval, and send result.
6. Add repository/storage codecs only after domain tests establish valid and invalid states.
7. Add the `training-progress-reporter` agent spec and output validation after approval gates are deterministic.

Keep provider sends, portal writes, and customer message delivery outside the domain aggregate. The domain should decide what is true and what is allowed; workflow/tools execute approved side effects.
