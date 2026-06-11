# Training implication 04: Curriculum tracking

Purpose: define the operational and domain contract for tracking a pet's training curriculum from enrollment through session evidence, milestone state, outcome review, and parent follow-up. This is a modeling artifact for later Rust/domain cards; it does not authorize provider writes or member-facing automation.

## Assumptions

- Curriculum tracking is owned by `operations::training`; it composes pet, care, temperament, reservation, staff, workflow, policy, and agent surfaces without absorbing their responsibilities.
- A curriculum is versioned by location/program so a 2-week Stay and Study, daycare tutor package, group class, puppy kindergarten, private lesson, and AKC CGC prep can share units while preserving different milestone requirements.
- Trainer notes and media references may be operational evidence, but completion of a milestone or outcome claim is a staff/trainer assertion. AI can structure drafts from evidence; it cannot mark customer-visible progress as approved.
- When PetSuites-specific lesson taxonomies are unknown, use an extensible `CurriculumUnit`/`BehaviorGoal` model with local policy validation rather than raw strings or hard-coded prose.

## 1. Operational story

### Trigger

Curriculum tracking starts when one of these events occurs:

- A training enrollment becomes `Scheduled` or `InProgress`.
- A trainer opens a session/task and records evidence for an enrolled pet.
- A staff member imports or reconciles legacy notes from a provider record.
- A manager daily brief detects missing progress evidence, stale milestone state, or an overdue parent follow-up.
- An agent extracts structured milestone drafts from approved trainer notes and routes them for review.

### Actors

- Trainer: records session evidence, updates milestone state, documents outcome readiness, and recommends homework.
- Lead trainer or manager: approves sensitive outcomes, overrides curriculum plans, resolves escalations, and approves customer-visible summaries when required.
- Front desk or customer-care staff: schedules sessions/classes, routes incomplete reports, and sends approved follow-ups.
- Pet parent/customer: receives approved progress reports/homework; does not directly mutate curriculum state.
- AI agent: drafts structured progress, missing-evidence alerts, and parent-message text; never finalizes milestone completion or sends member-facing content without approval.
- Provider/storage adapters: persist curriculum/enrollment/progress facts and external references after domain policies approve the draft.

### Inputs

- `operations::training::EnrollmentId`, `entities::PetId`, `entities::CustomerId`, `entities::LocationId`, optional `entities::ReservationId`.
- Selected `operations::training::ProgramKind`, `ProgramDuration`, `PackagePolicy`, and location `Contract`.
- `operations::training::CurriculumId`/curriculum version and ordered milestone templates.
- Trainer identity: `entities::StaffId`, trainer qualification, assignment, and session/class reference.
- Evidence: typed trainer note, scorecard, attendance/session observation, homework instruction, media/external reference, and optional parent practice feedback.
- Care/temperament context needed to determine whether progress language is safety-sensitive.
- Approval context: actor, automation level, review gate, audit subject, and provider/tool draft reference.

### Decisions

- Which curriculum version applies to this enrollment and whether it matches the program kind/duration.
- Whether the next unit/milestone can be introduced, practiced, demonstrated, deferred, or escalated.
- Whether evidence is sufficient for trainer-visible state only, staff-approved parent summary, or outcome documentation.
- Whether a progress report is overdue based on package/session cadence and follow-up policy.
- Whether a milestone update creates a staff task, manager review task, parent homework draft, package/re-enrollment opportunity, or no downstream action.
- Whether AI-generated structure is acceptable as a draft or unsafe because it infers behavior/medical/safety meaning from incomplete notes.

### Outputs

- Updated `operations::training::CurriculumProgress` aggregate with per-milestone state and evidence references.
- `operations::training::ProgressReport` draft or approved report, scoped to an enrollment/session/program.
- `operations::training::OutcomeDocumentation` draft when the final curriculum/outcome threshold is met.
- `operations::StaffTask` for missing evidence, trainer review, manager approval, parent follow-up, or escalation.
- `workflow::WorkflowEvent` and `workflow::RecommendedAction` values for internal task/message/status drafts.
- `entities::AuditEvent` recording actor, subject, action, review gate, source evidence, and approval decision.
- Safe `operations::training::ParentFollowUp` or `workflow::RecommendedAction::DraftMessage` only when approval boundaries are preserved.

### Success state

A curriculum-tracking workflow succeeds when:

- The enrollment references one active curriculum version compatible with its program and location.
- Each milestone has a typed state, evidence trail, actor, timestamp, and review boundary.
- Customer-visible progress/homework/outcome summaries are either explicitly approved or remain internal drafts.
- Missing, stale, contradictory, or safety-sensitive evidence produces typed review tasks instead of silent progression.
- Storage and workflow events can reconstruct who changed what, from which evidence, under which approval gate.

### Failure and exception states

- `CurriculumMissing`: enrollment has no compatible curriculum version.
- `CurriculumVersionMismatch`: persisted progress references a retired or incompatible curriculum version.
- `MilestoneTransitionRejected`: requested transition skips required evidence, tries to regress without reason, or conflicts with program policy.
- `EvidenceInsufficient`: note/media/scorecard is missing, empty, unbounded, or not attributable to a trainer/session.
- `SensitiveOutcomeRequiresReview`: behavior, safety, medical, incident, or CGC-readiness language cannot be member-facing without trainer/manager review.
- `TrainerAssignmentRequired`: no trainer is assigned to the enrollment/session being updated.
- `ReservationContextRequired`: Stay and Study or boarding/daycare tutor progress cannot be attached to a free-floating context when the program requires a reservation.
- `ApprovalBoundaryViolation`: an agent/provider/tool attempts to mark progress approved, send a report, or mutate provider records without the required human approval.
- `DuplicateEvidenceIgnored`: idempotent import sees the same external evidence reference and avoids double-counting.

## 2. Domain types to add or refine

### Existing types to preserve/refine

- `operations::training::Contract`: keep as the location/program contract but refine curriculum from a loose `Vec<CurriculumUnit>` into a versioned plan reference plus requirements.
- `operations::training::CurriculumUnit`: keep the stable enum surface, add extension/newtype support only if location policy needs local units.
- `operations::training::ProgressTracking`: treat as a tracking strategy; do not overload it as the current progress state.
- `operations::training::Outcome`: keep as a semantic outcome catalog; pair it with `OutcomeDocumentation` and review state before customer use.
- `operations::training::FollowUpCadence`: use it to decide due follow-up, not as proof a message may be sent.

### New/refined semantic paths

- `operations::training::curriculum::Id`
  - Stable curriculum-version identity. Invariant: non-empty/UUID-backed and distinct from enrollment/program IDs.
- `operations::training::curriculum::Version`
  - Monotonic version label or semantic revision. Invariant: cannot be empty; retired versions can be read for history but not assigned to new enrollments unless policy permits.
- `operations::training::Curriculum`
  - Aggregate: `{ id, version, location_id, program_kind, units, milestone_templates, approval_rules }`.
  - Invariant: active curriculum must have at least one unit and at least one milestone template.
- `operations::training::CurriculumPlan`
  - Enrollment-specific plan generated from `Curriculum` plus pet/program context.
  - Invariant: every planned milestone references a template and has an ordered position within the plan.
- `operations::training::curriculum::UnitSequence`
  - Non-empty ordered units/milestones; owns ordering rather than exposing raw `Vec` semantics to policies.
- `operations::training::CurriculumMilestone`
  - Entity/value object for one tracked checkpoint. Fields: milestone id, unit, label/goal, required evidence, status, last updated actor, review boundary.
- `operations::training::milestone::Id`
  - Stable milestone checkpoint identity, separate from template identity.
- `operations::training::MilestoneStatus`
  - `NotStarted`, `Introduced`, `Practicing`, `DemonstratedWithTrainer`, `NeedsParentPractice`, `Deferred`, `EscalatedForReview`, `ApprovedComplete`.
  - Invariant: `ApprovedComplete` requires staff/trainer approval and sufficient evidence.
- `operations::training::MilestoneTransition`
  - Requested semantic state transition with actor, reason, evidence refs, and approval context.
- `operations::training::MilestoneTransitionDecision`
  - `Accepted`, `Rejected { reason }`, `NeedsReview { gate }`, `NoOpAlreadyCurrent`.
- `operations::training::ProgressEvidence`
  - `TrainerNote { note }`, `Scorecard { score }`, `AttendanceObserved`, `ParentHomeworkAssigned { instruction }`, `ParentPracticeFeedback { note }`, `PhotoOrVideoReference { media_id }`, `ExternalProviderNote { provider, id }`.
  - Invariant: evidence is attributable and cannot be empty; external references are provider/id pairs, not raw URLs in the domain core.
- `operations::training::ProgressNote`
  - Trimmed non-empty bounded text; debug/redaction policy should avoid leaking behavior/sensitive details.
- `operations::training::HomeworkInstruction`
  - Trimmed non-empty bounded parent-facing instruction; construction does not imply send approval.
- `operations::training::TrainerScore`
  - Bounded numeric or rubric enum score owned by training; invariant: valid scale/rubric only.
- `operations::training::CurriculumProgress`
  - Enrollment aggregate with plan id, milestone states, evidence index, report cadence, and audit revision.
  - Invariant: cannot contain milestone states outside the assigned plan; cannot mark customer-visible completion without approval boundary.
- `operations::training::ProgressReport`
  - Draft/approved report for a session/program interval. Invariant: member-facing content is gated by `ApprovalBoundary` or `policy::ReviewGate`.
- `operations::training::OutcomeDocumentation`
  - Staff-approved outcome claim with evidence refs and reviewer. Invariant: CGC readiness and behavior-sensitive outcomes require trainer or manager review.
- `operations::training::CurriculumTrackingPolicy`
  - Owns milestone transition legality, evidence sufficiency, due-report detection, and review-gate selection.
- `operations::training::CurriculumRepository`
  - Better public path: `operations::training::curriculum::Repository`; loads curriculum versions/templates.
- `operations::training::progress::Repository`
  - Loads/saves enrollment progress and appends evidence/report records.

## 3. Relationship map

### Entities and aggregates

- `operations::training::Enrollment` owns the training commitment and references the assigned `CurriculumPlan`.
- `operations::training::Curriculum` owns canonical program/location curriculum requirements.
- `operations::training::CurriculumProgress` owns enrollment-specific milestone state and evidence references.
- `operations::training::ProgressReport` and `OutcomeDocumentation` are append-only documentation artifacts tied to an enrollment and evidence trail.
- `entities::Pet`, `entities::Customer`, `entities::Location`, and optional `entities::Reservation` remain adjacent identities/context, not fields flattened into training strings.

### Value objects

- IDs: `training::EnrollmentId`, `curriculum::Id`, `milestone::Id`, `entities::{PetId, CustomerId, LocationId, StaffId, ReservationId}`.
- Text values: `ProgressNote`, `HomeworkInstruction`, `OutcomeSummary`, `BehaviorGoal`, `workflow::Summary`, `workflow::message::Body` at the boundary.
- Cadence/scalars: `SessionCount`, `SessionMinutes`, `DurationWeeks`, `TrainerScore`, `curriculum::Version`.
- Evidence refs: `workflow::external::{Provider, Id}` or a future media reference value; no raw provider tuples.

### Policies

- `training::CurriculumSelectionPolicy`: chooses a curriculum version for program/location/enrollment context.
- `training::CurriculumTrackingPolicy`: validates milestone transitions and evidence sufficiency.
- `training::ProgressReportPolicy`: decides draft/approval/send boundaries for progress reports.
- `training::OutcomeApprovalPolicy`: decides trainer versus manager review for outcome claims.
- `training::FollowUpPolicy` or existing `FollowUpService`: creates staff tasks/message drafts when cadence says progress/homework is due.

### Repositories/stores

- `training::curriculum::Repository`: active/retired curriculum versions and templates.
- `training::enrollment::Repository`: enrollment state and assigned curriculum plan.
- `training::progress::Repository`: append evidence, milestone transitions, progress reports, outcome docs.
- `training::trainer::ScheduleRepository`: trainer assignment/session context needed before accepting progress.
- Storage adapters convert provider/export rows into semantic domain types and reject invalid text, IDs, stale versions, or unapproved member-facing states.

### Workflow events

- `training::WorkflowEvent::CurriculumPlanAssigned` or `workflow::WorkflowEventType` variant for assignment.
- `CurriculumEvidenceRecorded` when attributed evidence is appended.
- `MilestoneTransitionRequested` and `MilestoneTransitionApproved` for reviewable state changes.
- `ProgressReportDue`, `ProgressReportDrafted`, `ProgressReportApproved`.
- `OutcomeDocumentationReviewRequested` and `OutcomeDocumentationApproved`.

If the shared `workflow::WorkflowEventType` remains generic, preserve training meaning in the subject/action payload rather than stuffing it into free-form summaries.

### Staff tasks

Add training-specific `operations::StaffTaskKind` variants or a nested `operations::training::StaffTaskKind` converted into `operations::StaffTask`:

- `TrainingProgressEvidenceMissing { enrollment_id, milestone_id }`
- `TrainingMilestoneReview { enrollment_id, milestone_id, gate }`
- `TrainingOutcomeDocumentationReview { enrollment_id, outcome, gate }`
- `TrainingParentFollowUpDue { enrollment_id, cadence }`
- `TrainingCurriculumVersionMismatch { enrollment_id, expected, actual }`

### Agent specs/tools

- Agent spec: `agents::training_curriculum_tracker` or `agent::Spec { name: "training-curriculum-tracker", automation_level: DraftOnly }`.
- Allowed tools: read curriculum/enrollment/progress evidence; draft internal task; draft parent message; never direct-send or mark approved.
- Agent output schema should contain `DraftMilestoneTransition`, `EvidenceGap`, `DraftProgressReport`, and `ReviewGateRecommendation` rather than untyped prose.
- Provider tools should accept only approved domain drafts for writes/sends; tool policy denies approval boundary violations.

## 4. Interaction contract

Rust-like pseudo-signatures are illustrative; later cards should adapt to actual module layout while preserving ownership.

```rust
pub mod operations::training::curriculum {
    pub trait Repository {
        fn active_for(
            &self,
            location: entities::LocationId,
            program: training::ProgramKind,
        ) -> training::Result<training::Curriculum>;

        fn load_version(&self, id: Id, version: Version) -> training::Result<training::Curriculum>;
    }
}

pub mod operations::training::progress {
    pub trait Repository {
        fn load_for_enrollment(
            &self,
            enrollment: training::EnrollmentId,
        ) -> training::Result<training::CurriculumProgress>;

        fn append_evidence(
            &mut self,
            enrollment: training::EnrollmentId,
            evidence: training::ProgressEvidence,
        ) -> training::Result<training::EvidenceReceipt>;

        fn record_transition(
            &mut self,
            decision: training::AcceptedMilestoneTransition,
        ) -> training::Result<training::CurriculumProgress>;
    }
}
```

Behavior belongs on policies/services/aggregates, not helper functions:

```rust
impl training::CurriculumSelectionPolicy {
    pub fn assign_plan(
        &self,
        enrollment: &training::Enrollment,
        curriculum: &training::Curriculum,
    ) -> training::Result<training::CurriculumPlan>;
}

impl training::CurriculumTrackingPolicy {
    pub fn evaluate_transition(
        &self,
        progress: &training::CurriculumProgress,
        request: training::MilestoneTransition,
        context: training::TrackingContext,
    ) -> training::MilestoneTransitionDecision;

    pub fn evidence_sufficiency_for(
        &self,
        milestone: &training::CurriculumMilestone,
        evidence: &[training::ProgressEvidence],
    ) -> training::EvidenceSufficiency;

    pub fn due_progress_report(
        &self,
        progress: &training::CurriculumProgress,
        as_of: chrono::DateTime<chrono::Utc>,
    ) -> Option<training::ProgressReportDue>;
}

impl training::CurriculumProgress {
    pub fn request_transition(
        &self,
        request: training::MilestoneTransition,
        policy: &training::CurriculumTrackingPolicy,
        context: training::TrackingContext,
    ) -> training::MilestoneTransitionDecision;

    pub fn approved_milestones(&self) -> impl Iterator<Item = &training::CurriculumMilestone>;
}

impl training::ProgressReportPolicy {
    pub fn draft_report(
        &self,
        progress: &training::CurriculumProgress,
        evidence: &[training::ProgressEvidence],
        actor: entities::ActorRef,
    ) -> training::ProgressReportDecision;
}

impl training::OutcomeApprovalPolicy {
    pub fn evaluate_outcome_claim(
        &self,
        progress: &training::CurriculumProgress,
        outcome: training::Outcome,
        evidence: &[training::ProgressEvidence],
        actor: entities::ActorRef,
    ) -> training::OutcomeDocumentationDecision;
}
```

Orchestration service contract:

```rust
impl training::CurriculumTrackingService {
    pub fn record_trainer_evidence(
        &mut self,
        command: training::RecordTrainerEvidence,
    ) -> training::Result<training::RecordEvidenceOutcome>;

    pub fn request_milestone_transition(
        &mut self,
        command: training::RequestMilestoneTransition,
    ) -> training::Result<training::MilestoneTransitionOutcome>;

    pub fn draft_progress_report(
        &self,
        command: training::DraftProgressReport,
    ) -> training::Result<training::ProgressReportDraftOutcome>;
}
```

Expected service behavior:

- Load enrollment and assigned curriculum plan first; reject progress on unknown or incompatible plans.
- Verify trainer assignment/session context before accepting trainer-authored evidence.
- Append evidence idempotently before evaluating milestone transitions.
- Use `CurriculumTrackingPolicy` for transition legality and `ProgressReportPolicy`/`OutcomeApprovalPolicy` for member-facing boundaries.
- Emit workflow/staff-task/audit outputs as typed artifacts; provider adapters perform writes only from accepted/approved artifacts.

## 5. Review and approval contract

### Automation level

- Default automation: `policy::AutomationLevel::DraftOnly`.
- Deterministic policies may auto-classify missing evidence, stale reports, duplicate imports, and internal next-action drafts.
- AI may extract candidate milestone updates from trainer notes/media refs and draft parent-friendly summaries, but the output is evidence-linked and review-gated.

### Staff review gates

Staff/trainer review is required for:

- Marking a milestone `DemonstratedWithTrainer` or `ApprovedComplete`.
- Approving homework that will be sent to a customer.
- Approving a progress report for member-facing visibility.
- Reconciling imported provider notes when attribution or dates are ambiguous.
- Assigning evidence to a behavior-sensitive milestone.

### Manager review gates

Manager approval is required for:

- Safety-sensitive behavior conclusions, incidents, bite/aggression/reactivity language, or language affecting eligibility/care plans.
- CGC readiness claims when the policy requires lead trainer/manager signoff or evidence is ambiguous.
- Curriculum overrides, skipped mandatory milestones, or completion despite missing required evidence.
- Customer complaints, refund/credit/comp implications, or escalated parent communications.
- Any provider mutation that changes scheduled services/package entitlements rather than internal progress state.

### Audit trail

Every evidence append, milestone transition, report approval, outcome documentation, and customer-facing draft/send decision should record:

- actor (`entities::ActorRef`), time, location, enrollment, pet, and optional reservation/session.
- source evidence IDs and external provider references.
- old state, requested state, accepted state, and rejection/review reason.
- automation level and review gates applied.
- staff/manager approver when approval occurred.
- generated draft IDs if an agent or workflow produced text.

### Customer/member-facing boundaries

- No direct customer send from AI output.
- No outcome, behavior diagnosis, safety assurance, guarantee, or CGC-readiness claim without human approval.
- Parent homework/instructions are drafts until approved; sensitive care/behavior details must not be invented or softened.
- Internal progress may be summarized in a manager daily brief without customer visibility.
- Provider writes are tool-mediated and policy-gated; the domain returns drafts/decisions, not side effects.

## 6. Test contracts

Later implementation cards should add semantic tests before code changes. Suggested tests:

- `curriculum_requires_active_version_with_units_and_milestones`
  - Active `training::Curriculum` cannot be built without a program kind, non-empty unit sequence, and milestone templates.
- `curriculum_plan_preserves_program_location_and_version_identity`
  - Assigned plan carries typed `location_id`, `program_kind`, `curriculum::Id`, and `curriculum::Version`; IDs cannot be swapped with enrollment/pet IDs.
- `stay_and_study_curriculum_rejects_free_floating_progress_without_reservation_context`
  - Stay and Study progress requires the enrollment/reservation context policy says is mandatory.
- `milestone_transition_rejects_approved_complete_without_trainer_evidence`
  - `ApprovedComplete` requires sufficient attributed trainer evidence and review context.
- `milestone_transition_routes_safety_sensitive_behavior_to_manager_review`
  - Behavior/reactivity/safety evidence produces `NeedsReview { gate: ManagerApproval }` rather than auto-completion.
- `duplicate_external_evidence_reference_is_idempotent`
  - Re-importing the same provider/id evidence returns a no-op receipt and does not double-count milestone evidence.
- `progress_report_to_parent_requires_staff_approval_boundary`
  - Draft reports with member-facing body cannot become sendable without staff approval.
- `ai_curriculum_tracker_can_draft_but_cannot_approve_milestones`
  - Agent outputs candidate transitions/reports with review gates and cannot produce approved domain states.
- `outcome_documentation_preserves_evidence_and_reviewer`
  - Outcome claims include typed evidence refs and staff/manager approver.
- `cgc_readiness_requires_cgc_curriculum_unit_and_approval_gate`
  - CGC readiness cannot be documented unless the plan includes CGC prep and the appropriate review gate is satisfied.
- `curriculum_version_mismatch_creates_staff_task_not_silent_migration`
  - Loading progress against incompatible versions yields a typed staff/manager review task.
- `progress_note_and_homework_instruction_reject_empty_or_overlong_text`
  - Semantic text values trim inputs and reject invalid bounds.
- `training_progress_storage_roundtrip_preserves_approval_boundary`
  - Serialization/deserialization keeps milestone status, evidence refs, and approval state intact.
- `parent_follow_up_due_creates_internal_task_or_draft_message_without_send_permission`
  - Due cadence yields staff task/draft, not a sent message.

## 7. Integration notes for serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add/refine `operations::training` submodules or split into `operations/training/*.rs` if the module grows.
  - Add `Curriculum`, `CurriculumPlan`, `CurriculumProgress`, `CurriculumMilestone`, `MilestoneStatus`, `ProgressEvidence`, report/outcome decisions, and training-specific errors.
- `domain/src/workflow.rs`
  - Add typed workflow event/action support if existing generic events cannot preserve curriculum semantics.
- `domain/src/agents.rs` or `domain/src/agent.rs`
  - Add prompt/output schema values for curriculum tracker specs if agent contracts are modeled in-domain.
- `domain/src/entities.rs`
  - Add audit actions/subjects or staff-task kinds only if existing enums cannot represent training curriculum review truthfully.
- `domain/tests/petsuites_training_curriculum_tracking.rs`
  - Main semantic domain tests for curriculum/milestone/progress contracts.
- `domain/tests/training_agent_boundaries.rs`
  - Agent approval boundary tests.
- `storage/tests/training_progress_storage.rs` or existing storage test surface
  - Roundtrip/codec rejection tests once storage records exist.

### Migration/refactor risks

- Existing `operations::training::Contract.curriculum: Vec<CurriculumUnit>` is too loose for versioned tracking. Avoid breaking current service-contract tests in the first card; introduce richer plan types alongside the existing contract, then migrate constructors.
- `ProgressTracking` names a strategy, not progress state. Do not repurpose it as milestone status; add `MilestoneStatus`/`CurriculumProgress`.
- Existing `TrainingProgramDurationWeeks` and `training::DurationWeeks` accept any positive week count. Curriculum assignment for Stay and Study should depend on `StayAndStudyDuration` or policy validation to avoid unsupported curriculum versions.
- Trainer identity/qualification is not yet rich enough. Curriculum tracking can require `entities::StaffId` and `operations::StaffRole::Trainer` initially, but named trainer, CGC qualification, and behavior-case trainer should become semantic trainer requirement values before scheduling logic depends on them.
- Text evidence may include sensitive behavior/care details. Implement redaction/debug behavior consistently with `care` and `temperament` patterns before broad logging.
- Storage adapters must not deserialize approved/member-facing states from raw booleans. Use semantic enums and reject impossible approval boundary combinations.
- Do not create free-floating helpers such as `validate_milestone_update`; put behavior on `CurriculumTrackingPolicy`, `CurriculumProgress`, repositories, or services.

### Dependencies on other implications

- Enrollment/readiness implication: curriculum progress assumes a valid `Enrollment` with readiness resolved enough to schedule/start training.
- Trainer capacity/scheduling implication: milestone evidence should be accepted only for assigned trainer/session/class context.
- Progress reporting/outcome documentation implication: curriculum tracking feeds report/outcome artifacts but does not make customer send decisions by itself.
- Revenue/package follow-up implication: completion and overdue progress can create upsell/re-enrollment recommendations, but package/payment mutation remains outside curriculum tracking.
- Agent approval-boundary implication: AI extraction/drafting must share review-gate and audit semantics with the broader training automation contract.

Implementation posture: add the minimal domain contract first, prove it with semantic tests, then serialize/store it. Keep provider writes, customer sends, and AI execution as later adapter/tool cards behind review-gated domain decisions.
