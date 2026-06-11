# Training implication 06: Outcome documentation

Purpose: define the operational contract for documenting training outcomes after sessions/programs, preserving trainer evidence, approval boundaries, auditability, and safe parent follow-up. This is a modeling artifact for later Rust/domain cards, not an implementation patch.

Source context:

- `docs/domain/petsuites/training/service-domain-map.md`
- `domain/src/operations.rs`, especially `operations::training::{Contract, ProgressTracking, Outcome, TrainerAvailability, PackagePolicy, FollowUpCadence}`
- `domain/tests/petsuites_core_service_contracts.rs`
- `domain/tests/domain_quality_patterns.rs`

Assumption: PetSuites-specific outcome labels and scoring rubrics vary by location/trainer. The safe extensible model is a structured outcome-documentation aggregate with typed evidence, trainer/staff approval, and location/program policy references; free-text notes are allowed only as bounded semantic values attached to evidence/review state, not as final domain truth by themselves.

## 1. Operational story

### Trigger

Outcome documentation starts when one of these operational events occurs:

- A trainer completes a training session that requires a progress note, milestone update, scorecard, homework assignment, or parent-facing summary.
- A multi-session program reaches a configured milestone, program completion, CGC-readiness checkpoint, or trainer-review checkpoint.
- A manager/lead detects stale or missing documentation in the daily brief, enrollment queue, or trainer task list.
- An agent summarizes trainer notes and finds a draftable outcome claim, missing evidence, overdue parent follow-up, or ambiguous/safety-sensitive language.
- A customer asks what their pet learned, whether the pet is ready for a next class/CGC evaluation, or what homework should continue at home.

### Actors

- Trainer: authors observed evidence, milestone status, scorecard values, homework/coaching instructions, and outcome claims.
- Lead trainer or manager: approves ambiguous, CGC, safety-sensitive, escalation, or member-facing documentation when policy requires review.
- Front desk / customer care staff: may view approved documentation and send approved parent follow-up, but should not invent or finalize trainer outcome claims.
- Customer/parent: receives approved summary, homework, next-step recommendation, or re-enrollment prompt.
- AI agent: drafts structured summaries, detects missing evidence, flags review gates, and proposes staff tasks or draft messages. The agent never finalizes claims, marks approval, or sends member-facing documentation by itself.
- Workflow/tool layer: carries draft task/message/action packets and records audit events. Provider writes/sends remain approval-gated boundary actions.

### Inputs

- `operations::training::EnrollmentId` and typed links to `entities::{CustomerId, PetId, LocationId}` plus optional `entities::ReservationId` when training was attached to boarding/daycare.
- Program contract: `operations::training::Program`, `ProgramKind`, `Curriculum`, `CurriculumMilestone`, `ProgressTracking`, expected `Outcome`s, `FollowUpCadence`, and package/recurring-engagement policy.
- Trainer context: `entities::StaffId`, `operations::training::TrainerAssignment`, qualification requirement, named-trainer requirement, and location trainer policy.
- Evidence: trainer notes, milestone demonstrations, scorecard entries, media references, homework instructions, care/behavior caveats, parent-practice observations, and CGC-readiness rubrics.
- Adjacent facts: pet care profile, temperament/behavior observations, incident/escalation facts, reservation context, package usage, customer contact preference, and policy review gates.
- Prior documentation: previous progress reports, unresolved milestones, prior outcome documentation, open staff tasks, and previous parent follow-ups.

### Decisions

- Is the documentation a session-level progress report, a milestone update, a program-completion outcome, a CGC-readiness claim, a behavior/safety-sensitive note, or a parent follow-up draft?
- Does every claimed outcome have trainer-authored or trainer-approved evidence?
- Are required curriculum milestones present and in a terminal or explicitly deferred state?
- Is the wording safe for member-facing use, or does it include behavior diagnosis, guarantees, incident handling, medical/care restrictions, or ambiguous safety language?
- Which review gate applies: trainer approval, lead trainer approval, manager approval, parent-message approval, or unsafe-for-automation?
- Should the outcome create a follow-up task, homework draft, package/re-enrollment opportunity, CGC evaluation recommendation, or escalation task?
- Is storage/update allowed now, or should the workflow remain as a draft pending human review?

### Outputs

- `operations::training::OutcomeDocumentation` with outcome claims, evidence links, milestone state, summary, documented-by actor, review state, and audit metadata.
- `operations::training::ProgressReport` update or append-only progress event when a session-level note is recorded.
- `operations::StaffTask` for missing evidence, trainer review, manager review, parent follow-up due, package/re-enrollment review, or escalation.
- `workflow::RecommendedAction::DraftMessage` for approved-review pending parent communication, with `policy::ReviewGate` preserved.
- `operations::training::ParentFollowUp` or safe `UpsellOpportunity`/re-enrollment recommendation when policy allows staff review.
- `entities::AuditEvent` describing who drafted, reviewed, approved, changed, sent, or rejected each documentation artifact.

### Success state

Outcome documentation is successful when:

- The documentation references the correct enrollment, pet, customer, location, program, session/milestone, and trainer.
- Every outcome claim is backed by typed evidence and a review state that reflects the claim's risk.
- Required review gates are satisfied before any customer-visible summary, homework, outcome claim, or schedule/re-enrollment recommendation is sent.
- Missing/deferred milestones and limitations are explicit, not silently omitted.
- The artifact is append-only or versioned enough to preserve audit history.
- Follow-up tasks/messages are generated only as drafts or approved actions according to policy.

### Failure and exception states

- Missing evidence: create `OutcomeDocumentationDecision::NeedsTrainerEvidence` and a trainer task; do not finalize the outcome.
- Missing trainer identity or assignment mismatch: route to trainer/manager review before storing a final claim.
- Ambiguous CGC readiness: require lead trainer or manager approval; never treat AI confidence as readiness.
- Safety/behavior-sensitive language: require manager/lead trainer review and redact/route member-facing language until approved.
- Care/medical or incident conflict: route to care/manager review and link the relevant care/incident subject rather than hiding the conflict in notes.
- Customer-facing send requested without approval: return/send-denied policy result and preserve the draft.
- Conflicting prior documentation: create a review task and versioned correction path; do not overwrite history.
- Provider/storage decode error for raw notes/status strings: reject at boundary and require semantic conversion into typed values.

## 2. Domain types to add or refine

### New semantic IDs and bounded text values

- `operations::training::OutcomeDocumentationId`
  - Stable identity for a documentation artifact. Distinct from enrollment, reservation, progress report, and workflow event IDs.
- `operations::training::ProgressReportId`
  - Stable identity for session-level report artifacts when progress and final outcome documentation need separate lifecycles.
- `operations::training::OutcomeClaimId`
  - Stable identity for an individual claim inside a documentation artifact when claims can be reviewed independently.
- `operations::training::TrainerScore`
  - Validated rubric value, preferably an enum/range-backed newtype owned by the training module. It must not be a naked integer with implied scale.
- `operations::training::OutcomeSummary`
  - Non-empty bounded text. Construction only proves text shape; member-facing approval lives on `OutcomeReviewState`/`ApprovalBoundary`.
- `operations::training::ProgressNote`
  - Non-empty bounded trainer/staff note; debug/log output should redact sensitive details like care/temperament notes.
- `operations::training::HomeworkInstruction`
  - Non-empty bounded parent coaching instruction; unsafe for direct customer send until review permits.
- `operations::training::DocumentationRevisionReason`
  - Non-empty bounded reason or enum-backed value for corrections/versioning.

### Enums and aggregate types

- `operations::training::OutcomeDocumentation`
  - Aggregate for final or in-review program/session outcome documentation.
  - Invariants:
    - Has `EnrollmentId`, `PetId`, `LocationId`, `ProgramKind`/program ref, one or more `OutcomeClaim`s, documented-by actor, and review state.
    - Cannot be marked approved without an approver actor and approval timestamp/audit reference.
    - Cannot be member-facing unless `OutcomeReviewState::ApprovedForMemberFacingUse` or an equivalent approval boundary is present.
    - CGC-readiness or behavior-sensitive claims require trainer/lead/manager review by policy.

- `operations::training::OutcomeClaim`
  - One claim about what the pet achieved or still needs.
  - Shape:
    - `outcome: operations::training::Outcome`
    - `status: OutcomeClaimStatus`
    - `evidence: Vec<ProgressEvidence>`
    - `milestones: Vec<MilestoneId>`
    - optional `limitations: Vec<OutcomeLimitation>`
  - Invariant: positive/complete claims require at least one evidence item and a non-empty summary or linked milestone proof.

- `operations::training::OutcomeClaimStatus`
  - `Achieved`
  - `PartiallyAchieved { next_practice: HomeworkInstruction }`
  - `IntroducedNeedsPractice`
  - `Deferred { reason: OutcomeLimitation }`
  - `NotAssessed`
  - `EscalatedForReview { gate: TrainingReviewGate }`

- `operations::training::OutcomeLimitation`
  - `InsufficientSessions`
  - `PetStressOrFatigueObserved`
  - `CareOrMedicalReviewRequired`
  - `BehaviorSafetyReviewRequired`
  - `ParentPracticeRequired`
  - `EnvironmentalDistractionSensitivity`
  - `TrainerEvidenceMissing`
  - `LocationPolicyRequiresReview`
  - `Other(OutcomeLimitationNote)`

- `operations::training::OutcomeReviewState`
  - `DraftedByAgent { workflow_event_id: workflow::WorkflowEventId }`
  - `DraftedByStaff { actor: entities::ActorRef }`
  - `NeedsTrainerReview { gate: TrainingReviewGate }`
  - `NeedsManagerReview { gate: TrainingReviewGate }`
  - `ApprovedForInternalRecord { approved_by: entities::ActorRef }`
  - `ApprovedForMemberFacingUse { approved_by: entities::ActorRef }`
  - `Rejected { reason: DocumentationRevisionReason }`
  - `Superseded { by: OutcomeDocumentationId, reason: DocumentationRevisionReason }`

- `operations::training::DocumentationAudience`
  - `InternalTrainerRecord`
  - `ManagerReview`
  - `ParentSummaryDraft`
  - `ParentHomework`
  - `CgcReadinessRecord`
  - `PackageOrReEnrollmentRecommendation`

- `operations::training::OutcomeDocumentationDecision`
  - `RecordInternal { documentation: OutcomeDocumentation }`
  - `RequiresReview { draft: OutcomeDocumentation, gates: Vec<TrainingReviewGate> }`
  - `NeedsTrainerEvidence { missing: Vec<DocumentationEvidenceRequirement> }`
  - `UnsafeForAutomation { reason: AutomationUnsafeReason }`
  - `Rejected { reason: DocumentationRevisionReason }`

- `operations::training::DocumentationEvidenceRequirement`
  - `TrainerNoteRequired`
  - `MilestoneStatusRequired { milestone_id: MilestoneId }`
  - `ScorecardRequired`
  - `CgcRubricRequired`
  - `CareOrBehaviorReviewRequired`
  - `MediaOrObservationReferenceRequired`

### Existing types to refine

- Refine `operations::training::Outcome` from a coarse enum into the stable top-level outcome vocabulary. Keep variants such as `BasicManners`, `ReducedReactivity`, `CanineGoodCitizenReadiness`, and `OwnerHandlingPlan`, but require claim status/evidence/review context outside the enum.
- Refine `operations::training::ProgressTracking` into policy input rather than documentation state. It says what tracking is required; it does not prove documentation happened.
- Promote `operations::training::ProgressEvidence` from the service map into code before using AI-drafted documentation.
- Promote `operations::training::CurriculumMilestone` and `MilestoneStatus` before any final documentation depends on milestone completion.
- Promote `operations::training::ApprovalBoundary`/`TrainingReviewGate` so automation policy can be checked by type, not by string convention.

## 3. Relationship map between types

### Entities and value objects

- `OutcomeDocumentation` references `EnrollmentId`, `entities::PetId`, `entities::CustomerId`, `entities::LocationId`, optional `entities::ReservationId`, and one or more `OutcomeClaim`s.
- `OutcomeClaim` references `Outcome`, `OutcomeClaimStatus`, `ProgressEvidence`, `CurriculumMilestone`/`MilestoneId`, and `OutcomeLimitation`.
- `OutcomeSummary`, `ProgressNote`, `HomeworkInstruction`, `OutcomeLimitationNote`, and `DocumentationRevisionReason` are bounded semantic text values. They are not interchangeable strings.
- `TrainerScore` and CGC-specific rubric values are value objects owned by `operations::training`; they should not live in generic policy/helpers modules.

### Policies

- `operations::training::OutcomeDocumentationPolicy` owns whether a draft may become internal record, requires trainer/manager review, needs more evidence, or is unsafe for automation.
- `operations::training::ProgressReportPolicy` owns session report drafting and member-facing progress-report safety.
- `operations::training::CurriculumPolicy` owns required milestone evidence for each program type.
- `operations::training::RecurringEngagementPolicy` consumes approved outcome documentation and package usage to propose re-enrollment/package opportunities.
- `policy::{AutomationLevel, ReviewGate}` can remain the cross-domain automation vocabulary, but training-specific gates should be represented as `operations::training::TrainingReviewGate` and converted to generic policy gates only at workflow boundaries.

### Repositories/stores

- `operations::training::progress::Repository`
  - Appends `ProgressReport` and `OutcomeDocumentation` records.
  - Loads documentation by enrollment, pet, program, stale/missing status, or review state.
  - Preserves audit/version history rather than destructive overwrite.
- `operations::training::curriculum::Repository`
  - Loads milestone templates and required evidence rules.
- `operations::training::enrollment::Repository`
  - Loads enrollment/program/trainer/package context needed to validate outcome documentation.
- `operations::training::trainer::ScheduleRepository`
  - Optional relationship for verifying trainer assignment identity when outcome authoring requires the assigned/named trainer.
- Storage adapters convert raw provider rows/JSON into semantic documentation types and reject invalid scalars, empty notes, or unknown unsafe statuses.

### Workflow events, staff tasks, and audit

- Workflow subjects should point to `WorkflowSubject::External` provider IDs only at integration boundaries; domain decisions should use typed training IDs.
- Recommended actions may include internal tasks, draft messages, status/update drafts, or package/re-enrollment review tasks. They must preserve review gates.
- Staff tasks should gain training-specific kinds rather than hiding domain meaning in generic titles:
  - `TrainingOutcomeReview { enrollment_id }`
  - `TrainingMissingEvidence { enrollment_id }`
  - `TrainingParentFollowUpDue { enrollment_id }`
  - `TrainingCgcReadinessReview { enrollment_id }`
  - `TrainingDocumentationCorrection { documentation_id }`
- `entities::AuditEvent` records draft, review, approval, rejection, supersession, and customer-send events with typed subjects/actions/metadata.

### Agent specs/tools

- `agents::AgentPromptPacket<TrainingOutcomeDocumentationInput>` gives the agent only approved/read-safe context and asks for draft structure, gaps, and review recommendations.
- Agent output should be a proposed `OutcomeDocumentationDraft` or DTO that must pass deterministic `OutcomeDocumentationPolicy` before entering the domain aggregate.
- Tools may create task/message/provider-update drafts, but may not mark documentation approved or send parent-facing summaries without review proof.

## 4. Interaction contract

Rust-like pseudo-signatures are illustrative and should be adjusted to the final module split.

### Aggregate construction

```rust
impl operations::training::OutcomeDocumentation {
    pub fn builder() -> outcome_documentation::Builder<NeedsEnrollment, NeedsClaims, NeedsReview>;

    pub fn approve_for_internal_record(
        self,
        approver: entities::ActorRef,
        audit: entities::AuditEventId,
    ) -> training::Result<Self>;

    pub fn approve_for_member_facing_use(
        self,
        approver: entities::ActorRef,
        audit: entities::AuditEventId,
    ) -> training::Result<Self>;

    pub fn supersede(
        self,
        replacement: operations::training::OutcomeDocumentationId,
        reason: operations::training::DocumentationRevisionReason,
    ) -> training::Result<Self>;

    pub fn is_member_facing_send_allowed(&self) -> bool;
    pub fn unresolved_review_gates(&self) -> &[operations::training::TrainingReviewGate];
}
```

Behavior belongs on the aggregate because only the aggregate owns the combination of claims, evidence, review state, and audit identity.

### Claim construction

```rust
impl operations::training::OutcomeClaim {
    pub fn new(
        outcome: operations::training::Outcome,
        status: operations::training::OutcomeClaimStatus,
        evidence: Vec<operations::training::ProgressEvidence>,
    ) -> training::Result<Self>;

    pub fn requires_human_review(&self) -> bool;
    pub fn evidence_satisfies(&self, requirement: &DocumentationEvidenceRequirement) -> bool;
}
```

`OutcomeClaim::new` owns claim-level invariants: achieved/readiness claims cannot be built with no evidence; deferred/not-assessed claims must carry an honest status/limitation rather than pretending success.

### Outcome documentation policy

```rust
pub trait operations::training::OutcomeDocumentationPolicy {
    fn decide(
        &self,
        draft: operations::training::OutcomeDocumentationDraft,
        enrollment: &operations::training::Enrollment,
        program: &operations::training::Program,
        curriculum: &operations::training::Curriculum,
        prior_reports: &[operations::training::ProgressReport],
        care_profile: &entities::CareProfile,
        temperament: &entities::TemperamentProfile,
    ) -> operations::training::OutcomeDocumentationDecision;
}
```

The policy owns cross-aggregate decisioning. It should not write storage, send messages, or mutate provider records. It returns a semantic decision that workflow/application layers can route.

### Progress service orchestration

```rust
impl operations::training::ProgressService<R, C, E, A>
where
    R: operations::training::progress::Repository,
    C: operations::training::curriculum::Repository,
    E: operations::training::enrollment::Repository,
    A: entities::AuditRepository,
{
    pub fn draft_outcome_documentation(
        &self,
        command: operations::training::DocumentOutcomeCommand,
    ) -> training::Result<operations::training::OutcomeDocumentationDecision>;

    pub fn approve_documentation(
        &self,
        command: operations::training::ApproveOutcomeDocumentationCommand,
    ) -> training::Result<operations::training::OutcomeDocumentation>;

    pub fn create_parent_follow_up_draft(
        &self,
        documentation_id: operations::training::OutcomeDocumentationId,
    ) -> training::Result<workflow::RecommendedAction>;
}
```

The service owns orchestration because it composes repositories, policy, audit, and workflow draft creation. It should not own claim validation internals that belong on `OutcomeClaim`/`OutcomeDocumentation`.

### Repository contracts

```rust
pub trait operations::training::progress::Repository {
    fn append_progress_report(
        &self,
        report: operations::training::ProgressReport,
    ) -> training::Result<operations::training::ProgressReportId>;

    fn append_outcome_documentation(
        &self,
        documentation: operations::training::OutcomeDocumentation,
    ) -> training::Result<operations::training::OutcomeDocumentationId>;

    fn load_outcome_documentation(
        &self,
        id: operations::training::OutcomeDocumentationId,
    ) -> training::Result<operations::training::OutcomeDocumentation>;

    fn list_by_enrollment(
        &self,
        enrollment_id: operations::training::EnrollmentId,
    ) -> training::Result<Vec<operations::training::OutcomeDocumentation>>;

    fn list_requiring_review(
        &self,
        location_id: entities::LocationId,
    ) -> training::Result<Vec<operations::training::OutcomeDocumentation>>;
}
```

Repository behavior is persistence/query, not policy. It should not decide approval, rewrite evidence, or silently coerce unknown provider strings into valid domain states.

### Agent contract

```rust
pub struct operations::training::TrainingOutcomeDocumentationInput {
    pub enrollment: operations::training::EnrollmentSnapshot,
    pub program: operations::training::Program,
    pub required_milestones: Vec<operations::training::CurriculumMilestone>,
    pub trainer_evidence: Vec<operations::training::ProgressEvidence>,
    pub prior_reports: Vec<operations::training::ProgressReport>,
    pub policy_context: workflow::PolicyContext,
}

pub struct operations::training::TrainingOutcomeDocumentationDraft {
    pub proposed_claims: Vec<operations::training::OutcomeClaimDraft>,
    pub proposed_summary: operations::training::OutcomeSummary,
    pub missing_evidence: Vec<operations::training::DocumentationEvidenceRequirement>,
    pub recommended_gates: Vec<operations::training::TrainingReviewGate>,
    pub parent_follow_up_draft: Option<operations::training::ParentFollowUpDraft>,
}
```

Agent output is intentionally a draft. A deterministic policy must promote or reject it.

## 5. Review/approval contract

### Automation level

- Allowed automatically:
  - Detect missing outcome documentation from completed sessions/programs.
  - Summarize trainer-authored notes into a draft outcome structure.
  - Map evidence to proposed milestones and claims when confidence/evidence is shown as draft metadata.
  - Create internal draft tasks for trainer review, manager review, missing evidence, or parent follow-up due.
  - Recommend package/re-enrollment review based on approved outcomes.

- Staff/trainer review required:
  - Marking a milestone demonstrated, achieved, deferred, or not assessed.
  - Finalizing any outcome claim about skills, behavior progress, homework, or next training plan.
  - Approving parent-facing session/program summaries or homework instructions.
  - Correcting or superseding previously documented outcomes.

- Manager/lead trainer approval required:
  - CGC-readiness claims, unless location policy explicitly allows assigned-trainer approval.
  - Behavior/safety-sensitive outcomes such as reduced reactivity, bite/aggression context, handling restrictions, or safety assurances.
  - Conflicts with care profile, temperament, incident notes, or customer complaint/escalation state.
  - Overrides of missing evidence, trainer assignment mismatch, or disputed documentation.

- Never AI-only/member-facing:
  - Sending outcome documentation, homework, diagnoses, guarantees, CGC readiness, or safety assurances directly to a customer.
  - Rewriting provider/customer records as final outcome truth without human approval.
  - Downplaying incidents or care/behavior restrictions.
  - Inventing evidence, milestone completion, trainer observations, or approval.

### Review gates

Use explicit gates, not raw labels:

- `TrainingReviewGate::TrainerReview`
- `TrainingReviewGate::OutcomeDocumentationApproval`
- `TrainingReviewGate::ParentMessageApproval`
- `TrainingReviewGate::SafetyOrBehaviorReview`
- `TrainingReviewGate::ManagerApproval`
- `TrainingReviewGate::CareProfileReview`

Generic `policy::ReviewGate` may carry these to workflow/tool surfaces, but training code should retain training-specific semantics until the boundary conversion.

### Audit trail

Every material transition needs an audit event:

- Draft created, including actor (`Agent` or staff) and source evidence IDs.
- Review requested and why.
- Staff/trainer approval or rejection.
- Manager/lead approval for special gates.
- Parent-facing send approval and actual send/draft creation.
- Supersession/correction and reason.
- Storage/provider write draft creation and final provider mutation if later integration cards allow it.

Audit metadata should carry typed keys/values or typed fields, such as `enrollment_id`, `documentation_id`, `workflow_event_id`, `review_gate`, and `source_evidence_count`; it should not be the only place where domain state lives.

### Customer/member-facing boundaries

Parent-facing output is a separate boundary from internal documentation. An internal approved record may still be unsafe for customers if it contains staff-only behavior notes, care caveats, or operational language. The model should represent both:

- `ApprovedForInternalRecord`
- `ApprovedForMemberFacingUse`

Draft parent messages should include review policy and source documentation ID. Sending remains a tool/provider action and requires explicit approval proof.

## 6. Test contracts

Recommended tests for the later Rust implementation card:

### Domain construction and invariants

- `domain/tests/petsuites_training_outcome_documentation.rs::outcome_documentation_requires_enrollment_claims_documented_by_and_review_state`
  - Builder cannot create a record without typed enrollment context, at least one claim, actor, and review state.

- `domain/tests/petsuites_training_outcome_documentation.rs::achieved_outcome_claim_requires_typed_evidence`
  - `OutcomeClaimStatus::Achieved` with empty evidence is rejected.

- `domain/tests/petsuites_training_outcome_documentation.rs::partial_outcome_claim_preserves_homework_instruction_as_bounded_value`
  - `PartiallyAchieved` carries `HomeworkInstruction`; empty homework is rejected.

- `domain/tests/petsuites_training_outcome_documentation.rs::cgc_readiness_claim_requires_cgc_evidence_and_review_gate`
  - `Outcome::CanineGoodCitizenReadiness` cannot be approved AI-only or without required rubric/milestone evidence.

- `domain/tests/petsuites_training_outcome_documentation.rs::behavior_sensitive_outcome_routes_to_safety_or_manager_review`
  - Reduced-reactivity/behavior-sensitive limitations require review gate rather than direct member-facing approval.

- `domain/tests/petsuites_training_outcome_documentation.rs::member_facing_send_requires_member_facing_approval_not_internal_approval`
  - Internal record approval alone does not allow parent message send.

- `domain/tests/petsuites_training_outcome_documentation.rs::superseded_documentation_preserves_replacement_id_and_revision_reason`
  - Corrections are versioned/superseded with reason, not destructive overwrites.

### Policy and workflow contracts

- `domain/tests/training_outcome_policy_contracts.rs::policy_returns_needs_trainer_evidence_when_required_milestones_lack_evidence`
  - Missing milestone/evidence produces `NeedsTrainerEvidence` plus evidence requirements.

- `domain/tests/training_outcome_policy_contracts.rs::policy_promotes_agent_draft_only_to_requires_review_decision`
  - Agent draft remains `RequiresReview`; it is not an approved record.

- `domain/tests/training_outcome_policy_contracts.rs::approved_outcome_documentation_can_create_parent_follow_up_draft_with_review_gate`
  - Follow-up draft references the documentation and carries `ParentMessageApproval` unless pre-approved.

- `domain/tests/training_outcome_policy_contracts.rs::recurring_engagement_uses_approved_outcomes_without_customer_send_permission`
  - Re-enrollment/package opportunity is staff task/draft only.

### Cross-module relationship contracts

- `domain/tests/training_outcome_relationship_contracts.rs::outcome_documentation_links_customer_pet_location_enrollment_and_optional_reservation_ids_without_swapping`
  - Typed IDs prevent accidental customer/pet/reservation/location swaps.

- `domain/tests/training_outcome_relationship_contracts.rs::care_profile_or_temperament_conflict_blocks_member_facing_outcome_claim`
  - Care/temperament conflicts become review gates or limitations.

- `domain/tests/training_outcome_relationship_contracts.rs::training_missing_evidence_creates_semantic_staff_task_kind`
  - Missing evidence produces a training-specific staff task kind, not only a generic title.

- `domain/tests/training_outcome_relationship_contracts.rs::audit_events_record_draft_review_approval_and_send_transitions`
  - Audit history covers each lifecycle transition with typed subjects/actions/metadata.

### Storage/serialization contracts

- `storage/tests/training_outcome_documentation_storage.rs::outcome_documentation_roundtrips_without_losing_review_state_or_evidence`
  - JSON/storage conversion preserves claims, evidence, review state, and audit refs.

- `storage/tests/training_outcome_documentation_storage.rs::training_outcome_codecs_reject_empty_summary_notes_homework_and_revision_reason`
  - Boundary decoding rejects empty bounded text.

- `storage/tests/training_outcome_documentation_storage.rs::unknown_provider_outcome_status_decodes_to_review_required_or_error_not_success`
  - Unknown external statuses cannot become approved outcome claims.

### Agent boundary contracts

- `domain/tests/training_agent_boundaries.rs::agent_can_draft_outcome_documentation_but_cannot_mark_it_approved`
  - Agent output cannot set approved states.

- `domain/tests/training_agent_boundaries.rs::agent_summary_preserves_missing_evidence_and_review_gate_recommendations`
  - Agent drafts expose gaps and gates; they do not hide uncertainty.

- `domain/tests/training_agent_boundaries.rs::agent_parent_follow_up_draft_carries_source_documentation_and_message_review_policy`
  - Draft message has source doc ID and review policy before any send.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add/refine `operations::training` types first if the crate remains single-file.
  - Prefer splitting later into `domain/src/operations/training/{mod.rs,outcome_documentation.rs,progress.rs,curriculum.rs,enrollment.rs,error.rs}` if the project moves away from the current monolithic operations file.
- `domain/src/policy.rs`
  - Only add generic review/automation vocabulary if training-specific gates need cross-domain conversion; avoid moving training semantics into generic policy too early.
- `domain/src/workflow.rs`
  - Add typed workflow/recommended-action relationships only if existing `RecommendedAction` cannot carry training task/message drafts with review gates.
- `domain/src/entities.rs`
  - Add typed audit subject/action variants if current audit vocabulary cannot represent training outcome documentation lifecycle.
- `domain/src/agents.rs` or `domain/src/agent.rs`
  - Add prompt packet input/output schema names for training outcome documentation drafts.
- `storage/tests/training_outcome_documentation_storage.rs`
  - Add after domain types settle; storage should follow domain contracts, not lead them.
- `domain/tests/petsuites_training_outcome_documentation.rs`
- `domain/tests/training_outcome_policy_contracts.rs`
- `domain/tests/training_outcome_relationship_contracts.rs`
- Existing tests may need updates:
  - `domain/tests/petsuites_core_service_contracts.rs`
  - `domain/tests/domain_quality_patterns.rs`

### Migration/refactor risks

- Current `operations::training::Outcome` is only an enum; do not overload it with evidence, approval, or text. Add `OutcomeClaim`/`OutcomeDocumentation` around it.
- Current `ProgressTracking` can be mistaken for actual progress state. Keep it as a requirement/policy input and add separate `ProgressReport`, `ProgressEvidence`, and `MilestoneStatus` state.
- Current `TrainingProgramDurationWeeks` and `DurationWeeks` accept any positive number. Outcome documentation for Stay and Study should eventually depend on a validated `StayAndStudyDuration` or program policy so impossible program lengths do not leak into milestone expectations.
- Avoid using `workflow::RecommendedAction::DraftMessage` as proof of send approval. Drafting and approved sending are different states.
- Do not hide training staff-task semantics in `workflow::task::Title`; add semantic staff task kinds when code changes reach task generation.
- Redaction/debug policy matters because progress notes, behavior limitations, and care conflicts may contain sensitive details.
- Append/version documentation instead of destructive update. Provider systems may overwrite records, but the domain should preserve audit and supersession semantics.

### Dependencies on other Training implications

- Depends on enrollment/readiness modeling for `EnrollmentId`, `EnrollmentStatus`, readiness gates, and trainer assignment context.
- Depends on curriculum/progress modeling for `CurriculumMilestone`, `MilestoneStatus`, `ProgressEvidence`, and score/rubric semantics.
- Depends on trainer capacity/assignment modeling for verifying the actor who can author/approve claims.
- Feeds parent follow-up and recurring-engagement implications: approved documentation is the source for homework, parent summaries, package/re-enrollment recommendations, and CGC next steps.
- Feeds agent/workflow boundary implementation: agent specs need draft-only output schemas and deterministic policy promotion.

### Suggested implementation sequence

1. Add semantic tests for bounded text values, `OutcomeClaim`, `OutcomeReviewState`, and member-facing approval distinction.
2. Add `OutcomeDocumentation` aggregate and module-local `training::Error`/`Result` variants for missing evidence, invalid approval, unsafe member-facing send, and supersession errors.
3. Add policy tests and deterministic `OutcomeDocumentationPolicy` decisions before agent integration.
4. Add repository traits and storage roundtrip tests only after the aggregate shape is stable.
5. Add workflow/staff-task/agent draft contracts last, keeping all customer-facing sends behind review-gated tool boundaries.
