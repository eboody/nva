# PetSuites Training service domain map

Purpose: model the Training core service domain for the NVA/PetSuites AI-operations foundation. This is a contract map for later Rust/domain cards, not an implementation patch. It preserves semantic-code doctrine: domain meaning belongs in truthful module paths, invariants belong in newtypes/enums/builders/policies, and AI behavior must stay behind typed review/approval boundaries.

## Assumptions and bounded-context summary

Assumptions:

- Training is a revenue-bearing core service line, not merely a boarding/daycare add-on. Tutor sessions may be attached to daycare or boarding, and Stay and Study is operationally a training program embedded in a boarding stay, but training owns curriculum, trainer availability, progress, outcomes, parent coaching, and package engagement.
- PetSuites service language includes Drop-off Stay and Study programs, 2-, 3-, and 4-week programs, tutor sessions during daycare/boarding, group classes, puppy kindergarten, private lessons, and AKC Canine Good Citizen prep.
- The foundation should model location-specific configuration because trainer staffing, program availability, class schedules, package rules, and escalation thresholds vary by resort.
- This document intentionally states extensible contract shapes. Later implementation cards should write failing semantic API tests before changing code.

Bounded context:

`operations::training` owns the operational contract for selling, scheduling, delivering, documenting, and following up on pet training programs at a resort. It should compose identity and adjacent facts from `entities`, `care`, `temperament`, `money`, `payment`, `reservation`, `workflow`, and `agents`, but it should not become a catch-all for customer profiles, pet health, payment capture, or tool execution.

Training domain responsibilities:

- Service catalog: which training offerings a location sells and how each offering is packaged.
- Enrollment contract: whether a pet/customer/reservation is ready for the selected program, and which human review gates remain.
- Trainer capacity: which staff/role/trainer qualification is required, and whether booking should be confirmed, waitlisted, or routed to review.
- Curriculum plan: the unit sequence, milestones, homework/parent coaching expectations, and proof of progress.
- Progress reporting and outcome documentation: session notes, milestone evidence, scorecards, CGC readiness, behavior plans, and parent follow-up.
- Revenue continuity: package eligibility, recurring engagement, graduation/re-enrollment opportunities, and safe upsell recommendations.
- Automation boundaries: AI can draft, summarize, detect gaps, and recommend; staff/manager humans approve booking changes, outcome claims, safety-sensitive interpretations, and member-facing messages.

Non-responsibilities:

- `customer` owns customer contact identity and preferences.
- `entities::Pet`, `entities::CareProfile`, and `temperament` own pet/care/behavior facts.
- `reservation` and `tools` own external booking/update drafts and provider interaction boundaries.
- `money`/`payment` own monetary amounts, deposits, payment status, authorization, refunds, and provider references.
- `workflow`/`agents` own event/action packets and agent execution surfaces.

## Domain vocabulary

- Training offering: a sellable training service, such as Stay and Study, tutor session, group class, puppy kindergarten, private lesson, or AKC CGC prep.
- Program: an offering instantiated with a duration/curriculum/package shape. Stay and Study 2-week and 4-week variants are different program contracts even if they share the same offering family.
- Enrollment: the customer/pet/reservation commitment to a program, including readiness checks, trainer assignment, payment/package state, and review gates.
- Training session: one scheduled trainer interaction, either standalone, attached to boarding/daycare, or part of a class/program cadence.
- Curriculum unit: a named skill/behavior module the trainer is expected to teach and document.
- Milestone: a curriculum checkpoint with required evidence and a visible progress state.
- Progress report: a trainer-authored or staff-approved update that can be summarized for parent communication.
- Outcome: a documented training result, such as basic manners, owner handling plan, reduced reactivity support plan, or CGC readiness.
- Parent follow-up: safe post-session or post-program communication, homework, scheduling/rebooking prompt, or escalation to a trainer/manager.
- Trainer availability: a capacity decision over trainer qualification, schedule, named-trainer requirement, and location demand.
- Package/recurring engagement: prepaid sessions, board-and-train bundles, re-enrollment recommendations, and recurring class/private lesson cadence.

## Domain type inventory with semantic paths

Existing implemented surface in `domain/src/operations.rs`:

- `operations::ServiceOffering::Training { program: operations::TrainingProgram }`
- `operations::TrainingProgram::{StayAndStudy, TutorSession, GroupClass, PuppyKindergarten, PrivateLesson, AkcCanineGoodCitizenPrep}`
- `operations::TrainingProgramDurationWeeks`
- `operations::training::Contract`
- `operations::training::ProgramDuration`
- `operations::training::DurationWeeks`
- `operations::training::SessionCount`
- `operations::training::CurriculumUnit`
- `operations::training::ProgressTracking`
- `operations::training::Outcome`
- `operations::training::TrainerAvailability`
- `operations::training::PackagePolicy`
- `operations::training::FollowUpCadence`
- `operations::CoreServiceContracts::training`
- `operations::CoreServiceLine::Training`
- Related operational signals: `operations::RevenueOpportunityKind::TrainingConsultCandidate`, `operations::StaffRole::Trainer`, `operations::CapacityConstraintKind::TrainerAvailability`, `operations::CustomerCommunicationWorkflow::TrainingOptionsQuestion`, `operations::AiUseCase::TrainingOnboardingAssistant`, and `operations::OperatingFunction::Training`.

Recommended target public paths for the richer Training domain:

- `operations::training::Offering`
- `operations::training::OfferingKind`
- `operations::training::Program`
- `operations::training::ProgramKind`
- `operations::training::ProgramDuration`
- `operations::training::DurationWeeks`
- `operations::training::SessionCount`
- `operations::training::SessionMinutes`
- `operations::training::Enrollment`
- `operations::training::EnrollmentStatus`
- `operations::training::EnrollmentReadiness`
- `operations::training::ReadinessPolicy`
- `operations::training::TrainerAssignment`
- `operations::training::TrainerRequirement`
- `operations::training::TrainerAvailabilityDecision`
- `operations::training::Curriculum`
- `operations::training::CurriculumUnit`
- `operations::training::CurriculumMilestone`
- `operations::training::MilestoneStatus`
- `operations::training::ProgressReport`
- `operations::training::ProgressEvidence`
- `operations::training::Outcome`
- `operations::training::OutcomeDocumentation`
- `operations::training::ParentFollowUp`
- `operations::training::FollowUpCadence`
- `operations::training::PackagePolicy`
- `operations::training::RecurringEngagementPolicy`
- `operations::training::UpsellOpportunity`
- `operations::training::ApprovalBoundary`
- `operations::training::Repository` or split repositories under `operations::training::{offering,enrollment,progress}::{Repository}` if storage behavior grows independently.

Path guidance:

- Keep training-specific names under `operations::training`; avoid flat names such as `TrainingProgramContract`, `TrainingProgressStatus`, or `TrainerAvailabilityHelper` when `training::Program`, `training::ProgressReport`, and `training::AvailabilityPolicy` preserve meaning better.
- Keep `operations::TrainingProgram` only as a current top-level catalog enum while existing code uses it. Prefer migrating detailed behavior into `operations::training::{Program, OfferingKind, ProgramKind}`.
- Do not represent package, availability, curriculum, or follow-up rules as raw `String`, `bool`, or numeric fields.

## Existing Rust/domain surface to reuse or refactor

Reuse as stable foundation:

- `operations::training::Contract` already groups duration, curriculum, progress, outcomes, trainer availability, package, and follow-up into a semantic contract.
- `operations::training::DurationWeeks` and `SessionCount` already enforce positive scalar invariants through the existing `positive_scalar!` macro.
- `operations::training::ProgramDuration::{SingleSession, Weeks(DurationWeeks)}` is a good starting point for single-session versus multi-week offerings.
- `operations::training::CurriculumUnit` already names core curriculum concepts: `PuppyManners`, `LooseLeashWalking`, `Recall`, `ConfidenceBuilding`, and `CanineGoodCitizenPrep`.
- `operations::training::ProgressTracking` distinguishes attendance-only from session notes/milestones and trainer scorecards.
- `operations::training::Outcome` distinguishes domain outcomes rather than using note strings.
- `operations::training::TrainerAvailability` and `Contract::requires_named_trainer()` already expose a trainer-review invariant.
- `operations::training::PackagePolicy` and `FollowUpCadence` already encode recurring/package engagement and parent follow-up cadence.
- Existing tests in `domain/tests/petsuites_core_service_contracts.rs` and `storage/tests/core_service_contract_storage.rs` verify training contract construction, positive scalar rejection, standard PetSuites contracts, and JSON roundtrip behavior.

Refactor/expand before feature behavior depends on richer training semantics:

- Split the top-level `operations::TrainingProgram` from the richer `operations::training::Program`. The current top-level enum is useful as a catalog label, but real booking/progress behavior needs offering kind, duration, curriculum plan, package, and review rules in one semantic aggregate.
- Restrict Stay and Study durations to the known PetSuites program lengths. Current `TrainingProgramDurationWeeks` and `training::DurationWeeks` accept any positive week count; later code should add a `training::StayAndStudyDuration::{TwoWeeks, ThreeWeeks, FourWeeks}` or a policy-backed constructor that rejects unsupported durations.
- Add trainer qualification/role vocabulary before scheduling behavior branches on a raw staff ID or role string. `entities::StaffId` and `operations::StaffRole::Trainer` are identities/roles, not trainer qualification contracts.
- Promote progress reporting from `ProgressTracking` strategy to real `ProgressReport`, `CurriculumMilestone`, `MilestoneStatus`, and `ProgressEvidence` types before any AI report drafting or parent update behavior ships.
- Add `Enrollment`/`EnrollmentReadiness` before customer/pet/reservation/program eligibility is checked. Do not overload `entities::ReservationStatus` or `HardStop` with training-specific readiness.
- Add approval-boundary vocabulary before automating outbound parent messages, schedule confirmations, trainer assignments, or outcome claims.

## Required newtypes, enums, builders, policies, repositories, and services

### Newtypes and validated scalars

- `operations::training::OfferingId`: stable ID for a configured location offering; parsed from provider/storage IDs at boundaries.
- `operations::training::EnrollmentId`: stable ID for a training enrollment distinct from reservation ID.
- `operations::training::CurriculumId`: ID/version of a curriculum plan.
- `operations::training::MilestoneId`: ID for a curriculum checkpoint.
- `operations::training::SessionMinutes`: non-zero session duration.
- `operations::training::ClassCapacity`: non-zero class capacity; optional maximum can be policy-owned when class size matters.
- `operations::training::HomeworkInstruction`: non-empty bounded text; staff-approved before member-facing use.
- `operations::training::ProgressNote`: non-empty bounded text; may be sensitive and should have careful debug/redaction policy if stored.
- `operations::training::OutcomeSummary`: non-empty bounded text for staff-approved outcome documentation.
- `operations::training::BehaviorGoal`: non-empty bounded goal label or structured enum if behavior catalog stabilizes.

Invariants:

- IDs are non-empty or UUID/newtype-backed and cannot be accidentally swapped with customer/pet/reservation IDs.
- Session/package/duration scalars are positive.
- Member-facing text values are bounded and must carry staff approval state elsewhere; construction alone does not mean safe to send.
- Stay and Study duration must be one of 2, 3, or 4 weeks unless a location-specific policy explicitly permits an extension.

### Enums

- `operations::training::OfferingKind`
  - `StayAndStudy`
  - `TutorSession`
  - `GroupClass`
  - `PuppyKindergarten`
  - `PrivateLesson`
  - `AkcCanineGoodCitizenPrep`

- `operations::training::ProgramKind`
  - `DropOffStayAndStudy { duration: StayAndStudyDuration }`
  - `DaycareTutorSession`
  - `BoardingTutorSession`
  - `GroupClass`
  - `PuppyKindergarten`
  - `PrivateLesson`
  - `CanineGoodCitizenPrep`

- `operations::training::StayAndStudyDuration`
  - `TwoWeeks`
  - `ThreeWeeks`
  - `FourWeeks`

- `operations::training::EnrollmentStatus`
  - `Inquiry`
  - `NeedsPetProfileReview`
  - `NeedsTrainerReview`
  - `WaitlistedForTrainer`
  - `ReadyToSchedule`
  - `Scheduled`
  - `InProgress`
  - `Completed`
  - `Cancelled`
  - `Declined`

- `operations::training::EnrollmentReadiness`
  - `Ready`
  - `Blocked { reasons: Vec<ReadinessBlocker> }`
  - `NeedsHumanReview { gates: Vec<TrainingReviewGate> }`

- `operations::training::ReadinessBlocker`
  - `MissingCareProfile`
  - `MedicalOrMedicationReviewRequired`
  - `BehaviorReviewRequired`
  - `UnsupportedSpecies`
  - `VaccinesOrPolicyHardStop`
  - `NoTrainerCapacity`
  - `PaymentOrPackageRequired`
  - `ReservationContextRequired`

- `operations::training::TrainingReviewGate`
  - `TrainerReview`
  - `ManagerApproval`
  - `CareProfileReview`
  - `SafetyOrBehaviorReview`
  - `ParentMessageApproval`
  - `OutcomeDocumentationApproval`
  - `PaymentOrRefundApproval`

- `operations::training::TrainerRequirement`
  - `AnyCertifiedTrainer`
  - `NamedTrainer { staff_id: entities::StaffId }`
  - `CgcQualifiedTrainer`
  - `PuppyClassQualifiedTrainer`
  - `BehaviorCaseTrainer`

- `operations::training::TrainerAvailabilityDecision`
  - `Available { assignment: TrainerAssignment }`
  - `Waitlist { reason: TrainerCapacityReason }`
  - `Unavailable { reason: TrainerCapacityReason }`
  - `ManagerReviewRequired { reason: TrainerCapacityReason }`

- `operations::training::MilestoneStatus`
  - `NotStarted`
  - `Introduced`
  - `Practicing`
  - `DemonstratedWithTrainer`
  - `NeedsParentPractice`
  - `Deferred`
  - `EscalatedForReview`

- `operations::training::ProgressEvidence`
  - `TrainerNote { note: ProgressNote }`
  - `MilestoneCompleted { milestone_id: MilestoneId }`
  - `Scorecard { score: TrainerScore }`
  - `ParentHomeworkAssigned { instruction: HomeworkInstruction }`
  - `PhotoOrVideoReference { media_id: workflow::external::Id }`

- `operations::training::ApprovalBoundary`
  - `AutomationAllowed`
  - `StaffReviewRequired { gate: TrainingReviewGate }`
  - `ManagerApprovalRequired { gate: TrainingReviewGate }`
  - `MemberFacingSendRequiresApproval`
  - `UnsafeForAutomation { reason: AutomationUnsafeReason }`

### Builders and aggregates

- `operations::training::Offering::builder()`
  - Required: `location_id`, `kind`, `program`, `package`, `trainer_requirement`, `price_or_deposit_policy` if pricing is modeled in this aggregate.
  - Optional/default: curriculum, follow-up cadence, recurring engagement policy, member-facing availability label.
  - Invariant: an offering cannot be active without a valid program and trainer requirement.

- `operations::training::Program::builder()`
  - Required: `kind`, `duration`, `curriculum`, `progress_tracking`, `outcomes`.
  - Invariant: Stay and Study requires `StayAndStudyDuration`; tutor/private lesson allows single-session; CGC prep must include `CurriculumUnit::CanineGoodCitizenPrep` or a richer `CgcReadiness` unit.

- `operations::training::Enrollment::builder()`
  - Required: `customer_id`, `pet_id`, `location_id`, `program`, `status`, `readiness`.
  - Optional: `reservation_id` for boarding/daycare-attached training, `payment/deposit`, trainer assignment, package, follow-up plan.
  - Invariant: `Scheduled`/`InProgress` enrollment must have trainer assignment and no unresolved blocking readiness reasons.

- `operations::training::ProgressReport::builder()`
  - Required: `enrollment_id`, `pet_id`, `trainer_id`, `session_or_program_ref`, `evidence`, `approval_boundary`.
  - Invariant: member-facing report cannot be sent unless approval boundary permits or staff approval is recorded.

- `operations::training::OutcomeDocumentation::builder()`
  - Required: `enrollment_id`, `outcomes`, `summary`, `documented_by`, `review_gate`.
  - Invariant: outcome claims such as CGC readiness require trainer/staff review, not AI-only generation.

### Policies

- `operations::training::ReadinessPolicy`
  - Inputs: customer, pet, care profile, temperament profile, selected program, reservation context, payment/package state, location contract.
  - Output: `EnrollmentReadiness`.
  - Owns: training-specific readiness and review gates.
  - Does not own: vaccine policy internals, money capture, provider writes.

- `operations::training::TrainerAvailabilityPolicy`
  - Inputs: selected offering/program, requested window/session cadence, trainer qualification requirement, location capacity snapshot, current assignments.
  - Output: `TrainerAvailabilityDecision`.
  - Owns: trainer scheduling/capacity decision vocabulary.

- `operations::training::CurriculumPolicy`
  - Inputs: program kind, pet age/life stage when available, behavior goals, current milestones.
  - Output: curriculum plan/milestone requirements.
  - Owns: which curriculum units are required for each program type.

- `operations::training::ProgressReportPolicy`
  - Inputs: evidence, progress tracking contract, member-facing status, review gate.
  - Output: draftable report, staff review requirement, or unsafe-to-send decision.
  - Owns: parent update safety boundary for progress reports.

- `operations::training::RecurringEngagementPolicy`
  - Inputs: completed outcomes, package usage, follow-up cadence, customer preference, location offering availability.
  - Output: safe upsell/re-enrollment recommendation or no action.
  - Owns: high-value upsell and recurring engagement recommendations.

### Repositories and domain services

Use repository paths when persistence/query behavior arrives:

- `operations::training::offering::Repository`: load active offerings by location/program kind.
- `operations::training::enrollment::Repository`: load/save enrollment state and attach reservation/package references.
- `operations::training::trainer::ScheduleRepository`: query trainer availability/assignments.
- `operations::training::curriculum::Repository`: load curriculum versions and required milestone templates.
- `operations::training::progress::Repository`: append progress reports and outcome documentation.

Use explicit domain services for orchestration across aggregates/policies:

- `operations::training::EnrollmentService`: evaluates readiness and creates enrollment drafts; returns review-gated drafts, not provider writes.
- `operations::training::SchedulingService`: evaluates trainer availability and proposes session/class schedule drafts.
- `operations::training::ProgressService`: creates progress report drafts from trainer evidence and flags missing/outcome documentation.
- `operations::training::FollowUpService`: proposes parent follow-up tasks/messages after sessions/program completion.
- `operations::training::RevenueService`: detects safe package/re-enrollment opportunities.

## Relationships to adjacent modules

- Customer: `entities::CustomerId`, `entities::Customer`, `customer::{Name, Email, Phone}`, and `entities::ContactChannel` identify the parent and communication preference. Training should not store raw parent contact fields except as boundary snapshots.
- Pet: `entities::PetId`, `entities::Pet`, `entities::Species`, `pet::Name`, `entities::SpayNeuterStatus`, and `temperament` profile values inform eligibility, curriculum tailoring, safety review, and outcome wording.
- Reservation: `entities::ReservationId`, `entities::Reservation`, `entities::ServiceKind::Training`, `entities::AddOn`, `entities::HardStop`, and `reservation` policy values connect tutor sessions and Stay and Study to boarding/daycare stays. Training enrollment should reference a reservation when operationally required rather than pretending every training service is a standalone reservation.
- Care profile: `entities::CareProfile`, `care` medication/allergy/medical-condition/contact values, and medication review requirements determine whether a trainer can proceed or needs staff/manager review.
- Location: `entities::LocationId`, `entities::Location`, `entities::Brand::PetSuites`, location service capabilities, and location policy refs scope which offerings, trainers, and policies are available.
- Staff task: `operations::StaffTask`, `operations::StaffTaskKind`, `operations::StaffTaskAssignment`, `operations::StaffRole::Trainer`, and `operations::StaffTaskSource` should represent review/follow-up work. Add training-specific staff task kinds before stuffing training meaning into generic titles.
- Money/payment: `money::Money`, `money::MinorUnits`, `payment::Deposit`, `payment::DepositStatus`, and training `PackagePolicy` connect deposits, prepaid sessions, and board-and-train bundles. Domain services should create payment/authorization drafts; actual capture/refund remains provider/tool mediated and approval-gated.
- Workflow/agent modules: `workflow::WorkflowEvent`, `workflow::RecommendedAction`, `workflow::DraftTask`, `workflow::DraftMessage`, `agents` prompt packets, and `policy::{AutomationLevel, ReviewGate}` should carry training recommendations and review boundaries. Agents produce drafts and evidence summaries; deterministic policies decide allowed actions.

## AI-agent opportunities and approval boundaries

Automation allowed without member-facing side effects:

- Classify inbound questions as `operations::CustomerCommunicationWorkflow::TrainingOptionsQuestion` and draft internal next actions.
- Summarize active enrollments, missing progress reports, trainer capacity risks, and overdue parent follow-ups for a manager daily brief.
- Detect package/re-enrollment candidates from completed outcomes, unused sessions, lapsed engagement, or boarding/daycare context.
- Draft trainer task titles/descriptions for internal review, such as missing outcome documentation or parent follow-up due.
- Extract curriculum milestones from trainer notes into a structured draft when staff review remains required.
- Flag risk: unresolved behavior/care review, unsupported duration, missing trainer assignment, stale progress report, or package/payment mismatch.

Staff review required:

- Approving trainer assignment, trainer capacity decisions, class placement, or waitlist movement.
- Approving progress reports before they become parent-facing.
- Confirming curriculum milestone completion and outcome documentation.
- Sending homework or parent coaching instructions.
- Recording behavior-sensitive summaries or any language that could affect a pet's eligibility or care plan.

Manager approval required:

- Overriding trainer capacity/waitlist constraints.
- Waiving payment/package/deposit requirements, issuing refunds/credits, or comping sessions.
- Handling complaints, incidents, safety-sensitive behavior, or customer escalation.
- Claiming CGC readiness when evidence is ambiguous or staff policy says manager review is needed.
- Changing public service availability, pricing, package rules, or cancellation policy.

Unsafe/member-facing without explicit human approval:

- Sending training outcomes, behavior diagnoses, safety assurances, or guarantee-like claims directly to customers.
- Confirming bookings, schedule changes, payment capture, refunds, or package modifications through provider tools.
- Downplaying incidents, medical/care restrictions, reactivity, bite/aggression notes, or staff safety concerns.
- Inventing progress evidence or curriculum completion from incomplete notes.
- Using raw AI confidence as approval.

Boundary rule: AI may draft, recommend, summarize, and route. Typed policies plus staff/manager review decide whether a provider tool can mutate records or a message can reach a customer.

## Acceptance tests and contracts for later code cards

Recommended test file names are illustrative; later cards should choose focused tests and verify RED before implementation.

### Domain contract tests

- `domain/tests/petsuites_training_service_domain.rs::training_offering_kind_preserves_petsuites_service_surface`
  - Asserts every source offering exists as `operations::training::OfferingKind`/`ProgramKind` without raw strings.

- `domain/tests/petsuites_training_service_domain.rs::stay_and_study_duration_accepts_only_two_three_or_four_weeks`
  - Asserts known PetSuites durations are accepted and unsupported positive durations are rejected or require an explicit extension policy.

- `domain/tests/petsuites_training_service_domain.rs::training_program_builder_requires_duration_curriculum_progress_outcomes_and_trainer_requirement`
  - Asserts builders make incomplete programs unrepresentable or return semantic errors.

- `domain/tests/petsuites_training_service_domain.rs::cgc_prep_program_requires_cgc_curriculum_and_outcome`
  - Asserts CGC prep cannot be constructed without CGC-specific curriculum/outcome semantics.

- `domain/tests/petsuites_training_service_domain.rs::enrollment_cannot_be_scheduled_with_unresolved_readiness_blockers`
  - Asserts `Scheduled`/`InProgress` enrollment requires no blocking readiness reasons and requires trainer assignment.

- `domain/tests/petsuites_training_service_domain.rs::trainer_availability_policy_waitlists_when_named_trainer_is_required_but_unavailable`
  - Asserts named-trainer constraints produce waitlist/review decisions rather than silent booking.

- `domain/tests/petsuites_training_service_domain.rs::progress_report_to_parent_requires_staff_approval_boundary`
  - Asserts member-facing report drafts carry approval requirements unless approved evidence exists.

- `domain/tests/petsuites_training_service_domain.rs::outcome_documentation_preserves_trainer_evidence_and_review_gate`
  - Asserts outcome claims carry evidence and cannot be AI-only final claims.

- `domain/tests/petsuites_training_service_domain.rs::recurring_engagement_policy_recommends_package_follow_up_without_customer_send_permission`
  - Asserts upsell/re-enrollment opportunities create safe staff tasks/drafts, not direct sends.

### Cross-module relationship tests

- `domain/tests/training_relationship_contracts.rs::training_enrollment_links_customer_pet_location_and_optional_reservation_ids`
  - Asserts enrollment composes typed IDs and cannot swap customer/pet/reservation/location IDs.

- `domain/tests/training_relationship_contracts.rs::care_profile_and_temperament_review_block_training_readiness_when_required`
  - Asserts care/temperament facts become typed readiness blockers/review gates.

- `domain/tests/training_relationship_contracts.rs::boarding_or_daycare_tutor_session_requires_reservation_context`
  - Asserts tutor sessions attached to stays/daycare do not become free-floating records.

- `domain/tests/training_relationship_contracts.rs::payment_or_package_requirement_blocks_enrollment_without_payment_authorization_draft`
  - Asserts payment/package state is explicit and does not use raw booleans.

- `domain/tests/training_relationship_contracts.rs::training_follow_up_creates_staff_task_or_workflow_draft_with_review_gate`
  - Asserts follow-up produces typed `StaffTask`/workflow drafts and approval boundaries.

### Storage/serialization contract tests

- `storage/tests/training_contract_storage.rs::training_offering_records_roundtrip_between_storage_and_domain`
  - Asserts storage rows convert into semantic training domain types and back.

- `storage/tests/training_contract_storage.rs::training_codecs_reject_invalid_duration_session_count_and_empty_notes`
  - Asserts invalid scalars fail at decode boundaries.

- `storage/tests/training_contract_storage.rs::member_facing_progress_report_serialization_preserves_approval_boundary`
  - Asserts approval state is not lost across persistence.

### Agent/workflow contract tests

- `domain/tests/training_agent_boundaries.rs::agent_can_draft_training_follow_up_but_cannot_mark_customer_message_approved`
  - Asserts agent output is a draft/recommendation with review gate.

- `domain/tests/training_agent_boundaries.rs::agent_training_outcome_summary_requires_trainer_or_manager_review_for_member_facing_use`
  - Asserts outcome summaries are unsafe for direct send without approval.

- `domain/tests/training_agent_boundaries.rs::training_capacity_recommendation_routes_to_manager_when_overriding_waitlist_or_named_trainer_requirement`
  - Asserts capacity overrides are manager-gated.

## Implementation order for later code cards

1. Add focused tests for `operations::training::OfferingKind`, `ProgramKind`, and `StayAndStudyDuration`.
2. Expand `operations::training::Program`/`Offering` with builders and semantic construction errors.
3. Add enrollment/readiness contracts and policies that compose customer, pet, reservation, care, location, and payment/package state.
4. Add trainer availability/assignment vocabulary and staff-task/workflow draft outputs.
5. Add curriculum milestone/progress report/outcome documentation types with approval boundaries.
6. Add storage records and JSON roundtrip tests after the domain surface is stable.
7. Add agent prompt/action contracts only after deterministic policy tests prove approval gates.

Keep each implementation card narrow. Do not batch storage, provider tools, agent send behavior, and domain contracts in one change.
