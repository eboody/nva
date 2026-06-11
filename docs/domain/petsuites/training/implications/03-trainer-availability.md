# Training operational implication 03: Trainer availability

Purpose: define the domain contract for deciding whether a PetSuites training request can be staffed by an appropriate trainer, should be waitlisted, or must be routed to human review. This is a modeling artifact for later Rust/domain implementation cards; it does not prescribe provider writes or live scheduling behavior.

## Assumptions

- Trainer availability is more than a calendar slot. It combines location configuration, program kind, trainer qualification, named-trainer continuity, session cadence, current assignments, class capacity, and safety/review gates.
- A training enrollment may be standalone, attached to daycare/boarding as a tutor session, or embedded in a Stay and Study boarding program. Availability decisions must preserve that context instead of flattening every request into a generic appointment.
- When exact labor-scheduling details are unknown, the safest extensible model is: deterministic policy may recommend `available`, `waitlist`, or `review required`, but staff/manager approval confirms assignment, waitlist movement, capacity override, and any customer-facing schedule message.
- AI agents may summarize capacity risk, detect missing assignment, and draft internal tasks; they may not directly assign trainers, confirm bookings, override capacity, or message customers without typed review gates.

## 1. Operational story

### Trigger

Trainer availability evaluation starts when one of these events occurs:

- A customer asks about a training option and selects a program window.
- Staff creates or edits a training enrollment draft.
- A boarding/daycare reservation requests a tutor session or Stay and Study add-on.
- A group class/private lesson schedule is being built or changed.
- A location daily brief detects overbooked trainer capacity, missing trainer assignment, or a stale waitlist item.
- A named trainer becomes unavailable, changes role/qualification, or has an assignment conflict.

### Actors

- Customer/parent: requests training, preferred dates/times, trainer preference, and acceptable alternatives.
- Pet: the training subject; care/behavior facts may constrain trainer qualification or review gate.
- Front desk/reservation staff: create enrollment drafts, collect preferences, communicate approved availability choices.
- Trainer: owns teaching capacity, qualification fit, session cadence feasibility, and session-level acceptance.
- Lead trainer or manager: approves capacity overrides, named-trainer substitutions, behavior-sensitive placements, and waitlist exceptions.
- Scheduling/operations system: provides location, reservation, staff roster, assignment, waitlist, and capacity snapshots.
- AI operations assistant: drafts capacity summaries, flags risks, proposes staff tasks, and never mutates schedule/provider records directly.

### Inputs

- `operations::training::EnrollmentId` or enrollment draft details.
- `operations::training::ProgramKind` and `operations::training::OfferingId` scoped to `entities::LocationId`.
- Requested training window: date/time range, session cadence, preferred start date, class section, or boarding/daycare reservation span.
- Required trainer qualification: any certified trainer, named trainer, CGC-qualified trainer, puppy-class trainer, or behavior-case trainer.
- Optional named trainer preference/requirement and trainer continuity requirement.
- Pet/customer/reservation typed identities: `entities::{CustomerId, PetId, ReservationId, LocationId}`.
- Care/temperament readiness facts and hard stops from `care`, `temperament`, and `reservation`.
- Location capacity snapshot: trainer roster, qualifications, working shifts, current training assignments, group class capacity, waitlist, holidays/blackout windows.
- Package/payment context when capacity is tied to prepaid sessions, board-and-train bundles, or deposit requirements.

### Decisions

The availability policy answers:

1. Is the selected program active at this location and allowed in the requested context?
2. What trainer requirement applies to this program and this pet?
3. Does a qualified trainer have enough working capacity for the requested window/cadence?
4. If a named trainer is required or requested, is that trainer qualified, active, and available?
5. Does the request conflict with existing assignments, class capacity, boarding/daycare stay boundaries, or operational blackout windows?
6. Does pet care/behavior context require trainer review, lead-trainer review, or manager approval before schedule confirmation?
7. Should the system propose a concrete assignment, waitlist entry, alternate window, or review task?
8. Which outputs are internal-only drafts versus approved customer/member-facing availability statements?

### Outputs

- `operations::training::TrainerAvailabilityDecision`:
  - `Available { assignment }`
  - `Waitlist { entry, reason, alternatives }`
  - `Unavailable { reason }`
  - `TrainerReviewRequired { gate, reason }`
  - `ManagerReviewRequired { gate, reason }`
- `operations::training::TrainerAssignment` draft when a qualified trainer/window is selected.
- `operations::training::WaitlistEntry` draft when capacity is not presently available.
- Staff tasks routed to `operations::StaffRole::Trainer`, `operations::StaffRole::LeadStaff`, or `operations::StaffRole::Manager`.
- Workflow events/recommended actions for internal review and provider-tool drafts.
- Customer-message draft only after availability wording is explicitly marked as requiring approval.
- Audit trail explaining inputs, snapshot version, policy decision, review gates, and approving staff identity when applicable.

### Success state

A successful availability evaluation produces a typed, auditable decision that can be used by later enrollment/scheduling code without guessing:

- For low-risk requests with clear capacity, the system creates an internal assignment draft and routes it for trainer/staff confirmation.
- For constrained requests, the system creates a waitlist or alternate-window recommendation without pretending a booking is confirmed.
- For safety-sensitive, named-trainer, behavior-case, or override situations, the system routes review to the truthful human owner.
- No customer-facing promise is sent and no provider record is mutated until the required review/approval contract is satisfied.

### Failure and exception states

- Offering/program inactive at location: `Unavailable { reason: ProgramNotOfferedAtLocation }`.
- Missing location capacity snapshot: `TrainerReviewRequired { reason: CapacitySnapshotUnavailable }` or manager review for operationally urgent cases.
- Missing reservation context for Stay and Study/boarding tutor/daycare tutor: readiness blocker, not a free-floating assignment.
- No qualified trainer: waitlist or unavailable depending on location policy and requested date flexibility.
- Named trainer unavailable: waitlist when continuity is required; alternate trainer recommendation only with staff/customer approval.
- Trainer conflict or over-capacity: waitlist or manager review; no silent overbooking.
- Pet care/behavior hard stop: safety/behavior review before any trainer assignment.
- Group class full: waitlist/class-section alternatives; manager override only with typed approval.
- Payment/package state missing: availability may be estimated internally, but schedule confirmation remains blocked by enrollment/payment policy.
- Provider write failed after approval: preserve approved decision and emit a provider-sync failure event/task; do not mark enrollment scheduled.

## 2. Domain types to add or refine

### Refine existing training surface

- Replace the current coarse `operations::training::TrainerAvailability` contract enum with a richer pair:
  - `operations::training::TrainerRequirement`: the requirement carried by a program/offering/enrollment.
  - `operations::training::TrainerAvailabilityDecision`: the policy output for a specific request and capacity snapshot.
- Keep `Contract::requires_named_trainer()` as a compatibility query while migrating call sites, but make it delegate to richer requirement semantics once `Program`/`Offering` exist.
- Keep top-level `operations::TrainingProgram` only as a catalog label until `operations::training::ProgramKind` owns detailed behavior.

### New or richer value objects

- `operations::training::TrainerId` or reuse `entities::StaffId` inside a `trainer::Id` newtype if trainers need trainer-specific qualification/roster behavior. Prefer `operations::training::trainer::Id` only if trainer identity has semantics beyond generic staff identity; otherwise use typed `entities::StaffId` inside trainer-owned types.
- `operations::training::TrainerQualification`
  - `CertifiedTrainer`
  - `CgcEvaluatorOrCgcQualifiedTrainer`
  - `PuppyClassTrainer`
  - `BehaviorCaseTrainer`
  - `LeadTrainer`
- `operations::training::TrainerRequirement`
  - `AnyCertifiedTrainer`
  - `NamedTrainer { staff_id: entities::StaffId, continuity: TrainerContinuity }`
  - `QualifiedFor { qualification: TrainerQualification }`
  - `BehaviorCase { minimum: TrainerQualification }`
  - `ClassInstructor { qualification: TrainerQualification }`
- `operations::training::TrainerContinuity`
  - `Required`
  - `Preferred`
  - `NotRequired`
- `operations::training::TrainerAssignment`
  - Carries `enrollment_id`, `staff_id`, `requirement`, `window_or_cadence`, `assignment_status`, and `approval_boundary`.
  - Invariant: an assignment cannot be `Confirmed` when the availability decision is waitlisted/unavailable/review-required.
- `operations::training::AssignmentStatus`
  - `Draft`
  - `TrainerAccepted`
  - `StaffApproved`
  - `ConfirmedInProvider`
  - `DeclinedByTrainer`
  - `Cancelled`
- `operations::training::RequestedTrainingWindow`
  - Represents either a single session window, class section, multi-session cadence, or reservation-attached span.
  - Invariant: boarding/daycare-attached windows must reference the typed reservation context that bounds them.
- `operations::training::SessionCadence`
  - `SingleSession`
  - `Weekly { count: SessionCount }`
  - `ProgramSpan { duration: StayAndStudyDuration }`
  - `Custom { sessions: SessionCount, review_gate: TrainingReviewGate }`
- `operations::training::CapacitySnapshot`
  - Location-scoped, timestamped snapshot containing trainer roster, qualifications, shifts, assignments, class sections, waitlist counts, and blackout windows.
  - Invariant: policy decisions should carry or reference the snapshot version used for auditability.
- `operations::training::TrainerCapacityReason`
  - `ProgramNotOfferedAtLocation`
  - `NoQualifiedTrainerAtLocation`
  - `NamedTrainerUnavailable`
  - `RequestedWindowUnavailable`
  - `AssignmentConflict`
  - `ClassSectionFull`
  - `BlackoutWindow`
  - `CapacitySnapshotUnavailable`
  - `CareOrBehaviorReviewRequired`
  - `ManagerOverrideRequired`
  - `PaymentOrPackageBlocksConfirmation`
  - `ReservationContextRequired`
- `operations::training::WaitlistEntry`
  - Carries enrollment, requested window, requirement, reason, priority, alternates, and review boundary.
  - Invariant: waitlist entry cannot be customer-visible as a confirmed booking.
- `operations::training::AlternateTrainingWindow`
  - Carries proposed window/cadence, trainer fit, reason, and expiration/recheck policy if applicable.
- `operations::training::AvailabilityAuditRecord`
  - Captures policy input references, snapshot version, decision, generated tasks, reviewer/approver, and provider-write result.

### Explicit invariants

- `TrainerAssignment::confirmed` requires a qualified trainer, a capacity decision that permits assignment, no unresolved blocking readiness reasons, and an approval boundary that permits provider confirmation.
- `NamedTrainer { continuity: Required }` cannot be silently downgraded to `AnyCertifiedTrainer`; alternate trainer suggestions are recommendations requiring staff/customer approval.
- `BehaviorCase` and ambiguous safety/temperament cases route to trainer/lead/manager review, even when calendar capacity exists.
- `RequestedTrainingWindow` for tutor sessions attached to daycare/boarding must reference the reservation or enrollment context; it cannot be represented as only a loose date/time.
- `CapacitySnapshot` is location-scoped and stale snapshots cannot produce auto-confirmable decisions.
- `WaitlistEntry` and `AlternateTrainingWindow` are internal/customer-draft artifacts, not booking confirmations.
- Provider mutation and customer-facing availability messages require explicit approval/audit state separate from constructing the domain values.

## 3. Relationship map between types

### Entities and aggregates

- `operations::training::Offering` owns the trainer requirement default for a location/program.
- `operations::training::Program` owns the program kind, duration/cadence expectations, curriculum, outcomes, and default review gates.
- `operations::training::Enrollment` composes customer, pet, location, selected program/offering, readiness, package/payment state, optional reservation context, and current trainer assignment/waitlist state.
- `operations::training::TrainerAssignment` is an enrollment-owned scheduling artifact, not a generic staff record.
- `operations::training::WaitlistEntry` is the availability-side representation of unmet trainer capacity.
- `operations::StaffTask` represents human work created by availability decisions; add training-specific task kinds instead of encoding meaning in title strings.

### Value objects

- `entities::{CustomerId, PetId, LocationId, ReservationId, StaffId}` provide typed identity.
- `operations::training::{RequestedTrainingWindow, SessionCadence, AlternateTrainingWindow}` model time/cadence without leaking raw timestamps across the domain.
- `operations::training::{TrainerRequirement, TrainerQualification, TrainerContinuity}` model fitness to perform training work.
- `operations::training::{CapacitySnapshot, TrainerCapacityReason}` model why a request can or cannot be staffed.
- `operations::training::{AvailabilityAuditRecord, ApprovalBoundary}` model why an action was allowed or blocked.

### Policies

- `operations::training::TrainerAvailabilityPolicy` owns the deterministic availability decision.
- `operations::training::ReadinessPolicy` owns care/temperament/payment/reservation blockers that must be checked before confirmation.
- `operations::training::ProgramPolicy` or `OfferingPolicy` owns location-program active/inactive and requirement defaults.
- `operations::training::ApprovalPolicy` maps availability decisions to staff/manager/customer-message gates.

### Repositories and stores

- `operations::training::offering::Repository`: active offerings and default trainer requirements by location/program.
- `operations::training::enrollment::Repository`: enrollment state, assignment state, waitlist state, and typed links to reservations/packages.
- `operations::training::trainer::RosterRepository`: trainer staff identity, role, qualification, location, active/inactive state.
- `operations::training::trainer::ScheduleRepository`: shifts, existing assignments, class sections, blackout windows, and current capacity snapshot.
- `operations::training::availability::AuditStore`: append-only audit records for decisions, approvals, and provider-write outcomes.
- Storage layer should convert external/provider rows into these semantic values immediately; no domain policy should branch on raw provider status strings.

### Workflow events, staff tasks, and agent specs/tools

- Workflow events:
  - `training.availability.requested`
  - `training.availability.evaluated`
  - `training.assignment.review_required`
  - `training.waitlist.created`
  - `training.assignment.approved`
  - `training.assignment.provider_sync_failed`
- Staff task kinds to add/refine:
  - `ReviewTrainingTrainerAssignment`
  - `ApproveTrainingCapacityOverride`
  - `ContactCustomerAboutTrainingWaitlist`
  - `ResolveMissingTrainingCapacitySnapshot`
  - `ReviewBehaviorCaseTrainerFit`
- Agent specs/tools:
  - `training-capacity-brief` agent may read enrollment/roster/schedule snapshots and draft manager/trainer tasks.
  - `training-lead-conversion` agent may suggest available program options but must mark customer-facing availability language as approval-required.
  - Allowed tools: read-only portal/reservation/schedule reads, draft-task creation, draft-message creation.
  - Forbidden tools/actions: direct booking confirmation, trainer assignment provider write, capacity override, customer send, refund/payment modification.

## 4. Interaction contract

Rust-like pseudo-signatures below show ownership, not exact implementation syntax.

```rust
pub mod operations::training::trainer {
    pub struct Profile {
        pub staff_id: entities::StaffId,
        pub location_id: entities::LocationId,
        pub qualifications: Vec<TrainerQualification>,
        pub active: bool,
    }

    pub trait RosterRepository {
        fn active_trainers_at(
            &self,
            location: entities::LocationId,
        ) -> training::Result<Vec<Profile>>;

        fn trainer_profile(
            &self,
            staff_id: entities::StaffId,
        ) -> training::Result<Option<Profile>>;
    }

    pub trait ScheduleRepository {
        fn capacity_snapshot(
            &self,
            location: entities::LocationId,
            window: &RequestedTrainingWindow,
        ) -> training::Result<CapacitySnapshot>;
    }
}
```

```rust
pub struct TrainerAvailabilityPolicy;

impl TrainerAvailabilityPolicy {
    pub fn evaluate(
        &self,
        enrollment: &Enrollment,
        offering: &Offering,
        requested_window: RequestedTrainingWindow,
        snapshot: &CapacitySnapshot,
    ) -> TrainerAvailabilityDecision;
}
```

Behavior ownership:

- `Offering` owns default program requirement lookup: `offering.trainer_requirement_for(&program_kind)`.
- `TrainerRequirement` owns qualification matching: `requirement.is_satisfied_by(&trainer::Profile)`.
- `RequestedTrainingWindow` owns reservation-span/cadence validity: `window.is_allowed_for(&enrollment)`.
- `CapacitySnapshot` owns conflict and open-capacity queries: `snapshot.available_trainers_for(&requirement, &window)`.
- `TrainerAvailabilityPolicy` owns final decision vocabulary and review/waitlist routing. It should not directly save assignments, mutate provider systems, or send messages.
- `SchedulingService` orchestrates repositories and policies, returning drafts/events.

```rust
pub struct SchedulingService<Offerings, Enrollments, Roster, Schedule, Audit> { /* repos */ }

impl<Offerings, Enrollments, Roster, Schedule, Audit>
    SchedulingService<Offerings, Enrollments, Roster, Schedule, Audit>
{
    pub fn evaluate_trainer_availability(
        &self,
        enrollment_id: training::EnrollmentId,
        requested_window: RequestedTrainingWindow,
    ) -> training::Result<AvailabilityEvaluation>;

    pub fn approve_assignment(
        &self,
        assignment_id: training::AssignmentId,
        approval: training::AssignmentApproval,
    ) -> training::Result<workflow::RecommendedAction>;
}
```

`AvailabilityEvaluation` should contain:

- `decision: TrainerAvailabilityDecision`
- `audit_record: AvailabilityAuditRecord`
- `staff_tasks: Vec<operations::StaffTask>`
- `provider_draft: Option<tools::ProviderMutationDraft>` or equivalent later boundary type
- `customer_message_draft: Option<workflow::DraftMessage>` with `ReviewGate::CustomerMessageApproval`

Precise prose contract:

1. Loading: scheduling service loads enrollment, offering/program, readiness, and capacity snapshot through semantic repositories.
2. Validation: enrollment/window/program invariants run before availability policy; invalid context returns semantic `training::Error`.
3. Policy: availability policy returns a decision without side effects.
4. Review mapping: approval policy maps decision to required gates/tasks.
5. Persistence: enrollment repository may save a draft assignment/waitlist state; audit store appends the decision.
6. Boundary actions: provider writes and customer messages are represented as drafts/recommended actions until approvals are present.

## 5. Review and approval contract

### Automation level

Allowed automatically:

- Compute deterministic availability decisions from typed snapshots.
- Detect no-capacity, named-trainer-unavailable, class-full, stale-snapshot, missing-assignment, and review-required states.
- Create internal draft staff tasks for trainer/manager review.
- Draft alternate-window or waitlist communication text with customer-message approval gate.
- Produce manager daily brief snippets about trainer utilization and capacity risks.

Staff/trainer review required:

- Accepting a trainer assignment.
- Choosing between multiple qualified trainers when continuity, pet fit, or workload balance matters.
- Substituting an alternate trainer when a named trainer is preferred but not required.
- Moving a customer from waitlist to schedulable draft.
- Customer-facing availability explanation or schedule option message.

Manager approval required:

- Overriding trainer capacity, class capacity, blackout windows, stale snapshots, or location policy.
- Confirming an assignment despite readiness blockers or behavior/care concerns.
- Reassigning a behavior-case or safety-sensitive pet when trainer fit is ambiguous.
- Changing service availability, public schedule, pricing/package constraints, refund/credit/deposit handling.
- Any customer complaint/escalation tied to unavailable trainer capacity.

Forbidden without explicit human approval:

- Provider-tool booking confirmation or trainer-assignment mutation.
- Customer/member-facing message that promises a slot, trainer, outcome, or schedule change.
- Treating AI confidence or a draft recommendation as approval.
- Silent downgrade from named-trainer-required to any-trainer assignment.
- Overbooking trainer/class capacity.

### Audit trail

Every evaluation that creates a task, draft, waitlist entry, assignment draft, provider mutation draft, or customer-message draft should append an `AvailabilityAuditRecord` with:

- enrollment/offering/location identifiers;
- requested window/cadence and reservation context when applicable;
- requirement and candidate trainer fit;
- capacity snapshot version/timestamp;
- decision and reasons;
- generated staff tasks/workflow events;
- approval gate and approver identity when approved;
- provider draft/result reference when a boundary write is later attempted;
- customer-message draft/approval reference when customer-facing text is later sent.

## 6. Test contracts

Recommended semantic tests for later implementation:

- `trainer_requirement_matches_only_profiles_with_required_training_qualification`
  - CGC, puppy-class, and behavior-case requirements reject ordinary certified trainers unless they carry the required qualification.
- `named_trainer_required_waitlists_when_named_trainer_is_unavailable`
  - Named-trainer continuity cannot silently assign any certified trainer.
- `named_trainer_preferred_can_recommend_alternate_only_with_customer_message_review_gate`
  - Preferred named trainer may produce alternate-window/trainer suggestions, but outbound wording remains review-gated.
- `trainer_availability_uses_location_scoped_capacity_snapshot`
  - A trainer at another location or stale/unscoped snapshot cannot satisfy availability.
- `boarding_tutor_session_window_must_fit_reservation_span`
  - Reservation-attached tutor sessions cannot be scheduled outside the boarding/daycare reservation context.
- `stay_and_study_assignment_requires_program_span_capacity`
  - Multi-week Stay and Study requires capacity across the program span, not merely one open appointment.
- `class_section_full_creates_waitlist_instead_of_assignment`
  - Group class capacity produces a waitlist decision unless manager override is present.
- `behavior_case_trainer_fit_routes_to_review_even_when_calendar_slot_exists`
  - Care/temperament safety concern produces trainer/manager review gate before assignment confirmation.
- `capacity_override_requires_manager_approval_audit_record`
  - Overbooking or blackout override cannot produce provider mutation without manager approval in the audit trail.
- `availability_decision_can_create_staff_task_but_not_customer_send_permission`
  - Internal tasks are allowed; customer message drafts carry `ReviewGate::CustomerMessageApproval`.
- `stale_or_missing_capacity_snapshot_blocks_auto_confirmable_assignment`
  - Missing/stale snapshot yields review/unavailable, not `Available`.
- `provider_sync_failure_preserves_approved_assignment_draft_and_emits_staff_task`
  - Boundary failure does not erase the domain decision or falsely mark the enrollment scheduled.
- `assignment_confirmed_requires_no_unresolved_readiness_blockers`
  - Availability cannot bypass enrollment readiness/payment/package/care blockers.

## 7. Integration notes for the later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Expand `operations::training` with trainer requirement, qualification, assignment, waitlist, capacity snapshot, decision, reason, audit, and policy types.
  - Consider splitting `operations::training` into submodules once it grows: `offering`, `enrollment`, `trainer`, `availability`, `curriculum`, `progress`, `error`.
- `domain/src/policy.rs`
  - Add training-specific review gates or map to existing `ReviewGate::{ManagerApproval, BehaviorReview, CustomerMessageApproval}`; avoid generic string reasons.
- `domain/src/workflow.rs`
  - Add workflow/recommended-action shapes for availability evaluated, assignment review, waitlist, provider sync failure, and draft customer message.
- `domain/src/agent.rs` and/or `domain/src/agents.rs`
  - Add/extend training capacity agent spec with read/draft-only tools and default review gates.
- `domain/src/entities.rs`
  - Reuse `StaffId`, `LocationId`, `ReservationId`, `CustomerId`, `PetId`; add only if trainer-specific IDs or staff task kinds are missing.
- `domain/tests/petsuites_training_trainer_availability.rs`
  - New focused domain contract tests named above.
- `domain/tests/training_agent_boundaries.rs`
  - Tests proving AI can draft tasks/messages but cannot approve assignments/sends.
- `storage/tests/training_contract_storage.rs` or later storage module tests
  - Roundtrip decision/assignment/waitlist/audit records only after domain types stabilize.

### Migration and refactor risks

- Existing `operations::training::TrainerAvailability` is a contract-level requirement, not a decision. Rename/refactor carefully to avoid conflating requirement with evaluated capacity.
- `TrainingProgramDurationWeeks` and `training::DurationWeeks` currently accept any positive weeks. Stay and Study availability should depend on `StayAndStudyDuration::{TwoWeeks, ThreeWeeks, FourWeeks}` or an explicit location extension policy.
- Avoid raw `StaffRole::Trainer` as proof of qualification. Role is necessary but insufficient for CGC, puppy class, named trainer, and behavior-case constraints.
- Avoid creating free-floating helpers such as `check_trainer_available(...)`; behavior should live on requirement, snapshot, policy, scheduling service, or repositories.
- Do not place provider write behavior in `TrainerAvailabilityPolicy`; keep deterministic domain decisions separate from boundary tools.
- Member-facing text should not be encoded as a plain string that appears approved because it exists. Draft/approval state must travel with it.
- Storing snapshots/decisions needs versioning or timestamp semantics; otherwise audit cannot explain why a now-busy trainer was once recommended.
- If storage rows currently encode service/program/trainer fields as strings, the code card should implement boundary conversion with semantic errors instead of letting raw strings into domain policy.

### Dependencies on other implications

- Enrollment/readiness implication: availability can recommend a trainer only after readiness blockers are represented; final confirmation depends on no unresolved blockers.
- Curriculum/progress implication: trainer qualification may depend on program/curriculum units such as CGC readiness or behavior goals.
- Package/recurring engagement implication: prepaid package/deposit state may block confirmation even when trainer capacity exists.
- Parent follow-up/progress communication implication: waitlist/alternate-window/customer messaging must use the same approval boundary as other parent-facing training communications.
- Storage serialization card: should wait until requirement/decision/assignment/waitlist/audit types are stable enough for roundtrip tests.

## Implementation stance

The first Rust card should be narrow: introduce semantic types and failing tests for requirement matching, named-trainer waitlisting, location-scoped snapshot decisions, and customer-message review gates. Do not batch provider writes, storage migrations, agent execution, and full scheduling integration into the same change.
