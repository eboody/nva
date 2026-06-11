# Boarding implication 06: Medication / feeding / behavior notes

Purpose: model how Boarding turns medication, feeding, and behavior notes into truthful, reviewable stay obligations. This is a domain contract for later Rust cards. It does not authorize live care decisions, customer messaging, reservation changes, or medical/behavior approvals.

Assumptions:

- The source of truth for pet master data remains `entities::Pet`, `entities::CareProfile`, `entities::MedicationInstruction`, and `entities::TemperamentProfile`; Boarding consumes snapshots and creates stay-scoped obligations rather than duplicating the master profile.
- Medication, feeding, and behavior instructions are safety-sensitive. An AI agent may detect gaps, summarize, draft internal tasks, and prepare handoff packets, but staff or manager review is required before check-in completion when instructions are missing, ambiguous, newly changed, conflicting, or safety-relevant.
- Free-text notes are allowed at intake boundaries because provider systems and customer forms often provide free text. The domain core should immediately promote them into redacted semantic values, review outcomes, staff tasks, and audit evidence rather than passing raw strings through policies.
- Medication administration is never inferred from a vague note. When dosage, schedule, route, storage, or authorization is unknown, the safe extensible model is `VetClarificationRequired` or `ManagerReviewRequired` plus an internal staff task.

## 1. Operational story

### Trigger

The workflow starts whenever a Boarding stay enters a phase where care instructions must be trusted:

- pre-arrival checklist for a requested or confirmed boarding reservation;
- check-in staff review before the pet is accepted into a room/suite/condo;
- customer or staff edits to medication, feeding, allergies, medical conditions, temperament, or behavior notes;
- daily shift handoff for an active stay;
- playtime/upgrade recommendation that depends on feeding, medication, or behavior state;
- Pawgress Report or customer-message draft that wants to reference care facts.

The primary trigger should be a Boarding-owned care-review request, not a generic note parser:

```rust
operations::boarding::care::Policy::review_stay_care(
    &self,
    request: care::ReviewRequest,
    evidence: care::EvidenceSnapshot,
) -> care::ReviewOutcome;
```

### Actors

- Customer / pet parent: supplies or updates feeding instructions, medication details, emergency/veterinarian contacts, and behavior context.
- Front desk / check-in staff: verifies profile completeness, checks customer-provided changes, and decides whether check-in can proceed or needs a gate.
- Kennel technician: fulfills feeding, medication, potty-walk, housekeeping, and special-handling tasks during the stay.
- Lead staff / manager: approves ambiguous, conflicting, safety-sensitive, incident-related, or exception-based care decisions.
- Veterinarian / external clinic contact: clarifies medication or medical instructions when required by local policy.
- AI agent: summarizes evidence, detects missing/conflicting facts, drafts internal tasks and handoff packets, and drafts customer-facing text only in `DraftOnly` state.
- Deterministic domain policy: produces typed review outcomes, review gates, staff tasks, and audit events.

### Inputs

- `entities::Reservation` with `ServiceKind::Boarding`, stay dates, reservation status, requested add-ons, and hard stops.
- `entities::Pet` with `Species`, `SpayNeuterStatus`, `CareProfile`, and `TemperamentProfile`.
- `entities::CareProfile.feeding_instructions`, medications, allergies, medical conditions, emergency contact, and veterinarian contact.
- `entities::MedicationInstruction` values: medication name, dose, schedule, and `care::MedicationReviewRequirement`.
- `temperament::BehaviorObservation`, `GroupPlayObservation`, `TemperamentRating`, and redacted staff notes.
- Boarding service contract: included care features, medication support/add-on rules, handoff requirement, playtime/add-on catalog, Pawgress Report policy, and location-specific approval thresholds.
- Existing task evidence: completed/incomplete feeding or medication tasks, incident follow-up, staff review notes, and shift handoff state.
- Source metadata: customer-supplied, staff-entered, migrated/provider-imported, AI-extracted, or manager-approved.

### Decisions

The care review must answer at least these questions with typed decisions:

1. Feeding: Is there a complete feeding instruction for every boarded pet, and does it contain enough structured meaning for staff execution? Missing or conflicting feeding data creates a review task, not a silent default.
2. Medication: Are medication instructions absent, routine but staff-check-required, manager-review-required, or veterinarian-clarification-required? Each medication should be reviewed independently before creating daily administration tasks.
3. Behavior: Does temperament/behavior evidence allow the requested accommodation, playtime, handling, and report content? Behavior flags can block unsupervised group play while still allowing a staffed private enrichment path.
4. Care plan completeness: Can the stay move to `CareReviewed`, or does check-in remain blocked by `care::ReviewGate` values?
5. Task planning: Which staff tasks must exist, who should own them, when are they due, and what evidence completes them?
6. Communication boundary: Which care facts may be summarized internally, which may be included in a Pawgress Report after review, and which are sensitive enough to require manager approval before any customer-facing message?

### Outputs

- `operations::boarding::care::Plan` for the stay/pet, composed of typed feeding, medication, behavior, special-handling, and handoff requirements.
- `operations::boarding::care::ReviewOutcome` with explicit feeding/medication/behavior outcomes and a readiness state.
- Boarding-owned task intents mapped into `operations::StaffTask` drafts:
  - feeding review;
  - medication verification;
  - medication administration;
  - behavior or playgroup assessment;
  - special-handling note review;
  - emergency/vet contact verification;
  - manager escalation;
  - Pawgress Report sensitive-content review.
- `operations::boarding::handoff::Packet` entries that carry open care obligations into shift handoff.
- `workflow::WorkflowEvent` / audit entries with source evidence, review gate, actor, timestamp, and approval state.
- `operations::PetCareWatch` / `OperationsRisk::PetSafetyOrCareRisk` signals for manager daily briefs.
- Agent prompt packets or tool drafts constrained to internal read/summarize/task-create behavior unless a typed approval decision permits more.

### Success state

A stay is care-ready when:

- each boarded pet has an evaluated `boarding::care::ReviewOutcome`;
- feeding instructions are either complete or intentionally waived by an approved staff/manager decision that records the reason;
- each medication has an executable schedule/dose or a blocking clarification task;
- behavior flags are translated into playtime/special-handling decisions and staff tasks;
- all review gates required for check-in are closed by an authorized actor;
- open recurring care tasks are scheduled and visible in the handoff packet;
- customer-facing drafts are marked as draft/reviewed/approved according to their content risk.

### Failure and exception states

- Missing feeding instruction: creates `FeedingReview::MissingInstruction`, a front-desk or kennel-tech task, and a check-in/handoff warning.
- Conflicting feeding instruction: creates `FeedingReview::ConflictingInstruction` and blocks automation until staff resolves the conflict.
- Medication present with missing dose/schedule/authorization: creates `MedicationReview::VetClarificationRequired` or `ManagerReviewRequired`; do not create executable administration tasks until clarified.
- Medication changed during stay: creates a new care-review event and invalidates future administration tasks that relied on the old instruction.
- Behavior flag affecting safety or group play: creates `BehaviorReview::ReviewRequired`, blocks group-play recommendation, and may require manager approval for sensitive customer-facing text.
- Incident or injury signal: escalates to manager/safety review, blocks Pawgress Report auto-send, and requires an audit trail.
- Provider/imported note too vague to classify: preserved as redacted source evidence and routed to staff review; the AI must not infer operational instructions from it.
- Task completion evidence is free-text/media-only: staff task can be marked completed only by an authorized staff actor; AI may summarize evidence but not mark care complete.

## 2. Domain types to add or refine

### Boarding care module

- `operations::boarding::care::ReviewRequest`
  - Stay-scoped request for care review.
  - Fields: `location_id`, `reservation_id`, `pet_id`, `stay_range`, `requested_accommodation`, `requested_add_ons`, `review_phase`.
  - Invariants: service must be Boarding; stay range must be positive; one request is for one pet so per-pet care obligations cannot be lost in multi-pet reservations.

- `operations::boarding::care::ReviewPhase`
  - `PreArrival`, `CheckIn`, `ActiveStay`, `ShiftHandoff`, `CheckoutReport`, `ProfileChange`.
  - Invariant: customer-facing outputs are never approved from `PreArrival`/`ProfileChange` without explicit review.

- `operations::boarding::care::EvidenceSnapshot`
  - Snapshot of pet care profile, temperament profile, existing task evidence, source metadata, and local policy refs used by `care::Policy`.
  - Invariant: contains source provenance for every free-text note and every imported/provider fact.

- `operations::boarding::care::Plan`
  - Per-stay, per-pet fulfillment plan.
  - Fields: `feeding`, `medications`, `behavior`, `special_handling`, `tasks`, `handoff_items`, `approval_state`.
  - Invariant: cannot be `ReadyForCheckIn` while any blocking review gate remains open.

- `operations::boarding::care::ReviewOutcome`
  - Aggregates feeding, medication, and behavior review into `readiness`, `required_gates`, `staff_tasks`, `watchlist_reasons`, and `audit_events`.
  - Invariant: every required gate has a reason and a truthful owner.

- `operations::boarding::care::Readiness`
  - `ReadyForCheckIn`, `ReadyWithOpenNonBlockingTasks`, `BlockedForStaffReview`, `BlockedForManagerReview`, `BlockedForVetClarification`.
  - Invariant: blocking states must carry at least one typed review reason.

- `operations::boarding::care::ReviewGate`
  - Boarding-local gate that can map into `policy::ReviewGate`.
  - Variants: `FeedingInstructionReview`, `MedicationInstructionReview`, `MedicalSafetyReview`, `BehaviorReview`, `PlaytimeEligibilityReview`, `SensitiveCustomerMessageReview`, `ManagerApproval`, `VetClarification`.
  - Invariant: `VetClarification` and manager-only gates cannot be cleared by AI.

### Feeding types

- `operations::boarding::care::feeding::Plan`
  - Derived stay-scoped feeding obligation.
  - Fields: `instruction`, `source`, `review`, `tasks`.
  - Invariant: a concrete feeding task may only be scheduled when review is `Complete` or `StaffApproved`.

- `operations::boarding::care::feeding::InstructionSnapshot`
  - Wraps `care::FeedingInstruction` with source and last-reviewed metadata.
  - Invariant: redacted debug/display; raw customer/provider text stays behind the semantic value.

- `operations::boarding::care::FeedingReview`
  - `Complete`, `MissingInstruction`, `ConflictingInstruction`, `StaffReviewRequired`, `ManagerOrVetReviewRequired`, `WaivedByStaff { reason }`.
  - Invariant: waiver requires actor, reason, timestamp, and audit event.

- `operations::boarding::care::feeding::TaskKind`
  - `VerifyInstruction`, `PrepareMeal`, `FeedPet`, `RecordRefusalOrException`, `EscalateException`.
  - Invariant: `RecordRefusalOrException` should create a handoff item and may affect Pawgress Report review.

### Medication types

- `operations::boarding::care::medication::Plan`
  - One plan per medication or per medication group when local policy permits grouping.
  - Fields: `instruction`, `review`, `administration_tasks`, `storage_requirement`, `approval_state`.
  - Invariant: executable administration tasks require reviewed name, dose, schedule, and actor-authorized source.

- `operations::boarding::care::medication::InstructionSnapshot`
  - Wraps `entities::MedicationInstruction` with source provenance, profile version, and review metadata.
  - Invariant: medication name/dose/schedule use existing `care::*` redacted semantic newtypes; changes create a new snapshot rather than mutating past evidence.

- `operations::boarding::care::MedicationReview`
  - `NoMedication`, `StaffCheckRequired`, `Complete`, `ManagerReviewRequired`, `VetClarificationRequired`, `CannotAdminister { reason }`.
  - Invariant: anything other than `NoMedication`/`Complete`/approved `StaffCheckRequired` blocks check-in or creates an explicit blocking task.

- `operations::boarding::care::medication::AdministrationSchedule`
  - Domain schedule derived from `care::MedicationSchedule`.
  - Invariant: no zero/empty schedule; next due times must fall inside the stay range or be intentionally marked as pre/post-stay instructions.

- `operations::boarding::care::medication::TaskKind`
  - `VerifyMedication`, `AdministerDose`, `RecordDoseSkipped`, `RequestVetClarification`, `EscalateMedicationException`.
  - Invariant: skipped dose or exception creates `PetCareWatchReason::MedicationDue`/medical safety watchlist and manager handoff when policy requires.

### Behavior and special-handling types

- `operations::boarding::care::behavior::Plan`
  - Stay-scoped interpretation of `TemperamentProfile` for Boarding.
  - Fields: `review`, `playtime_eligibility`, `special_handling`, `handoff_notes`, `report_sensitivity`.
  - Invariant: behavior observations affect Boarding decisions through `behavior::Plan`; do not branch directly on scattered raw observations in task planners.

- `operations::boarding::care::BehaviorReview`
  - `NoKnownConcern`, `StaffObserveDuringStay`, `PlaytimeReviewRequired`, `SpecialHandlingRequired`, `ManagerReviewRequired`, `IncidentFollowUpRequired`.
  - Invariant: `ManagerReviewRequired` and `IncidentFollowUpRequired` block sensitive customer-facing communication until cleared.

- `operations::boarding::care::behavior::SpecialHandling`
  - `SeparateFromGroupPlay`, `TwoStaffHandling`, `QuietRoomPreferred`, `EscapeRiskPrecaution`, `FoodGuardingPrecaution`, `CatIndividualOnly`, `Other(Label)`.
  - Invariant: special-handling requirements generate handoff items and cannot be hidden behind staff-note free text.

- `operations::boarding::care::behavior::ReportSensitivity`
  - `Routine`, `CustomerReviewRecommended`, `ManagerReviewRequired`, `DoNotMentionUntilReviewed`.
  - Invariant: Pawgress/customer-message drafts must consult this before including medical/behavior facts.

### Audit and source types

- `operations::boarding::care::Source`
  - `CustomerProvided`, `StaffEntered`, `ProviderImport`, `ProfileSnapshot`, `TaskEvidence`, `AiExtractedDraft`, `ManagerApproved`.
  - Invariant: `AiExtractedDraft` cannot be used as executable care instruction without staff approval.

- `operations::boarding::care::ReviewedBy`
  - `Staff(entities::StaffId)`, `Manager(entities::StaffId)`, `Veterinarian(care::ContactRef)`, `PolicyOnly`.
  - Invariant: `PolicyOnly` is valid only for non-sensitive deterministic completeness checks.

- `operations::boarding::care::AuditEvent`
  - Records `WorkflowEventId`, source, outcome, actor, timestamp, and affected task/gate.
  - Invariant: review-state transitions must be audit-backed.

## 3. Relationship map

### Entities

- `entities::Reservation` anchors the stay lifecycle and status. Boarding care policy reads it and may recommend `MissingInfo`, `SpecialReview`, `CheckedIn`, or `Active` transitions, but agents do not mutate it directly.
- `entities::Pet` owns species, spay/neuter status, care profile, and temperament profile. Boarding creates a stay-specific projection from it.
- `entities::CareProfile` owns master feeding/medication/allergy/medical contact facts. Boarding `care::Plan` snapshots those facts for the stay.
- `entities::MedicationInstruction` remains the medication profile type. Boarding `medication::InstructionSnapshot` adds stay source/review/audit context.

### Value objects

- Existing redacted values: `care::FeedingInstruction`, `care::MedicationName`, `care::MedicationDose`, `care::MedicationSchedule`, `care::ReviewReason`, `temperament::StaffNote`, `temperament::BehaviorObservationLabel`.
- New Boarding values: `feeding::InstructionSnapshot`, `medication::AdministrationSchedule`, `behavior::SpecialHandling`, `care::ReviewGate`, `care::Source`, `care::Readiness`.
- Do not introduce generic `Note`; medication, feeding, behavior, source, and review notes have different invariants and owners.

### Policies

- `operations::boarding::care::Policy` owns stay-scoped care review.
- `policy::PlayEligibilityPolicy` remains the cross-service play decision surface; Boarding wraps its output in `boarding::playtime::Eligibility` / `behavior::Plan` for stay-specific fulfillment and handoff.
- `operations::boarding::agent::ApprovalPolicy` maps care outputs and proposed agent actions to `policy::AutomationLevel` and `policy::ReviewGate`.
- `operations::boarding::handoff::Policy` or planner owns whether open care obligations must appear in shift handoff.

### Repositories and stores

- `operations::boarding::care::Repository` reads care/temperament profile projections and writes stay-care plans, care-review outcomes, task projections, and handoff items.
- `operations::boarding::reservation::Repository` reads boarding reservation projections and proposes status/update drafts.
- `operations::boarding::report::Repository` stores Pawgress report drafts and their sensitivity/approval state.
- Storage modules should encode provider/imported notes as boundary records and convert into redacted semantic domain values before `care::Policy` sees them.

### Workflow events

- `care_review_requested`
- `feeding_instruction_missing`
- `feeding_instruction_approved`
- `medication_instruction_review_required`
- `medication_instruction_approved`
- `behavior_review_required`
- `care_plan_ready_for_checkin`
- `care_plan_blocked`
- `care_task_completed`
- `care_task_exception_recorded`
- `sensitive_report_content_review_required`

Events should carry typed IDs, source evidence refs, actor, review gate, and resulting readiness state.

### Staff tasks

Boarding-owned task kinds should map to `operations::StaffTaskKind` without losing Boarding semantics:

- `boarding::task::Kind::FeedingReview` -> `StaffTaskKind::Feeding { pet_id }` with `source = Reservation(reservation_id)` and review metadata.
- `boarding::task::Kind::MedicationVerification` -> `StaffTaskKind::MedicationAdministration { pet_id }` or `DocumentReview { pet_id }` depending on whether it is instruction review or dose administration.
- `boarding::task::Kind::BehaviorAssessment` -> `StaffTaskKind::PlaygroupAssessment { pet_id }` or `IncidentFollowUp { pet_id }`.
- `boarding::task::Kind::CareManagerEscalation` -> high/critical priority task with `StaffRole::Manager`.
- `boarding::task::Kind::PawgressSensitiveContentReview` -> `DailyUpdateDraft { reservation_id }` plus customer-message review gate.

### Agent specs and tools

- Agent specs: `boarding-pre-arrival-care-review`, `boarding-shift-handoff-draft`, `boarding-care-watchlist-brief`, `boarding-pawgress-draft-review`, `boarding-playtime-safety-screen`.
- Allowed read tools: reservation-read, pet-profile-read, care-note-read, task-read, policy-read.
- Allowed write/draft tools: task-create draft, handoff-packet draft, report draft, recommended-action draft.
- Disallowed without approval: mark medication/feeding complete, approve behavior/play eligibility, send customer messages, update reservation status, alter live medication instructions, hide care exceptions from reports.

## 4. Interaction contract

### Policy contracts

```rust
pub trait operations::boarding::care::Policy {
    fn review_stay_care(
        &self,
        request: care::ReviewRequest,
        evidence: care::EvidenceSnapshot,
    ) -> care::ReviewOutcome;

    fn feeding_review(
        &self,
        request: &care::ReviewRequest,
        instruction: Option<feeding::InstructionSnapshot>,
    ) -> care::FeedingReview;

    fn medication_review(
        &self,
        request: &care::ReviewRequest,
        medication: medication::InstructionSnapshot,
    ) -> care::MedicationReview;

    fn behavior_review(
        &self,
        request: &care::ReviewRequest,
        temperament: &entities::TemperamentProfile,
        play_decision: policy::PlayEligibilityDecision,
    ) -> care::BehaviorReview;
}
```

Behavior belongs on `care::Policy` because it is evaluating a stay-scoped Boarding care contract. It should not live as helper functions such as `validate_notes` or `parse_care_flags`.

### Planner contracts

```rust
impl operations::boarding::care::Plan {
    pub fn from_review(
        request: care::ReviewRequest,
        outcome: care::ReviewOutcome,
    ) -> care::Result<Self>;

    pub fn readiness(&self) -> care::Readiness;
    pub fn blocks_checkin(&self) -> bool;
    pub fn required_gates(&self) -> &[care::ReviewGate];
    pub fn handoff_items(&self) -> &[handoff::Item];
}

pub trait operations::boarding::workflow::Planner {
    fn plan_care_tasks(
        &self,
        plan: &care::Plan,
    ) -> Vec<boarding::task::Draft>;

    fn plan_handoff_items(
        &self,
        plan: &care::Plan,
    ) -> handoff::Packet;
}
```

`care::Plan` owns readiness and gate visibility. `workflow::Planner` owns task/handoff projection. Generic `operations::StaffTask` remains the cross-service task representation, not the source of Boarding care truth.

### Repository contracts

```rust
pub trait operations::boarding::care::Repository {
    fn load_evidence(
        &self,
        reservation_id: entities::ReservationId,
        pet_id: entities::PetId,
    ) -> care::Result<care::EvidenceSnapshot>;

    fn save_review_outcome(
        &self,
        outcome: &care::ReviewOutcome,
    ) -> care::Result<care::ReviewOutcomeId>;

    fn save_plan(
        &self,
        plan: &care::Plan,
    ) -> care::Result<care::PlanId>;

    fn append_audit_event(
        &self,
        event: care::AuditEvent,
    ) -> care::Result<()>;
}

pub trait operations::boarding::task::Repository {
    fn create_drafts(
        &self,
        drafts: Vec<boarding::task::Draft>,
    ) -> task::Result<Vec<operations::StaffTask>>;
}
```

Repositories store plans, projections, and audit events. They do not decide whether a vague medication note is safe; that is policy behavior.

### Approval and agent contracts

```rust
pub trait operations::boarding::agent::ApprovalPolicy {
    fn classify_care_action(
        &self,
        action: agent::CareAction,
        plan: &care::Plan,
    ) -> policy::AutomationRule;
}

pub enum agent::CareAction {
    SummarizeEvidence,
    DraftInternalTask,
    DraftHandoffPacket,
    DraftPawgressReport,
    MarkFeedingComplete,
    MarkMedicationAdministered,
    ApproveBehaviorForGroupPlay,
    SendCustomerCareMessage,
}
```

Expected mapping:

- Summarize evidence: `SafeToAutomate` if it remains internal and cites source refs.
- Draft internal task / handoff packet: `InternalTaskOnly`.
- Draft Pawgress Report: `DraftOnly`; sensitive medical/behavior/incident content adds customer-message or manager review.
- Mark feeding/medication complete: staff-only; never AI-only.
- Approve behavior for group play: staff/manager review required depending on evidence.
- Send customer care message: customer-message approval required; sensitive content may require manager approval.

## 5. Review and approval contract

### Automation level

- Safe to automate:
  - read-only summarization of existing care/profile/task evidence;
  - deterministic detection of missing feeding instructions, medication review requirements, and behavior flags;
  - creation of internal task drafts and handoff packet drafts;
  - manager brief/watchlist drafts.

- Internal task only:
  - feeding review tasks;
  - medication verification tasks;
  - behavior/playgroup assessment tasks;
  - vet clarification tasks;
  - manager escalation tasks.

- Draft only:
  - Pawgress Report text;
  - customer care-message text;
  - explanation of why a care gate blocks check-in or playtime;
  - summaries of medication/behavior exceptions.

- Staff approval required:
  - confirming feeding instructions are complete;
  - confirming routine medication instructions are executable;
  - marking feeding or medication tasks complete;
  - approving routine behavior/playtime decisions with no safety-critical flags;
  - approving non-sensitive Pawgress content.

- Manager or veterinarian clarification required:
  - medication dose/schedule/authorization ambiguity;
  - conflicting medical, allergy, or medication notes;
  - behavior evidence involving bite history, human selectivity, escape risk, incident history, or manager-review flags;
  - sensitive customer-facing messages about medical/behavior/incident decisions;
  - waiving required care data at check-in.

- Never automate:
  - changing live medication instructions;
  - approving medical/behavior safety;
  - hiding or deleting negative care facts;
  - sending sensitive customer-facing messages without a cleared review gate;
  - marking care complete from AI-generated evidence alone.

### Review gates

Every care gate should record:

- gate type;
- reason;
- affected reservation/pet/medication/task;
- source evidence refs;
- required role (`FrontDesk`, `KennelTechnician`, `LeadStaff`, `Manager`, veterinarian contact);
- opened/closed timestamps;
- actor who closed it;
- outcome and any follow-up task.

Absence of a gate is not approval. A gate is cleared only by a domain event from an authorized actor or by a deterministic policy where the type explicitly allows `PolicyOnly`.

### Audit trail

Audit records must preserve the transition, not raw sensitive text dumps:

- source: profile snapshot, customer update, staff note, task evidence, imported provider note, AI draft;
- decision: feeding/medication/behavior review outcome;
- action: task created, gate opened, gate closed, plan updated, report draft blocked/approved;
- actor and role;
- timestamp;
- previous/new readiness;
- redacted references to sensitive note values.

### Customer/member-facing boundaries

- Medication, behavior, incident, and safety details are sensitive. Agents may draft but cannot send.
- Pawgress Reports may mention routine care only after evidence and approval. Reports must not omit unresolved care risks or imply medication was administered unless the staff task has approved completion evidence.
- Customer requests to modify feeding/medication instructions create internal review tasks and do not automatically update executable stay plans.
- If customer text conflicts with staff/vet/provider records, the system routes to staff/manager review instead of choosing one.

## 6. Test contracts

Future implementation cards should add semantic tests with names like these:

- `boarding_care_review_missing_feeding_instruction_blocks_checkin_with_staff_task`
  - A Boarding pet with no `CareProfile.feeding_instructions` returns `FeedingReview::MissingInstruction`, a feeding review gate, and an internal staff task.

- `boarding_care_review_complete_feeding_instruction_schedules_feeding_task_without_manager_gate`
  - A redacted semantic `care::FeedingInstruction` with no conflict produces an executable feeding task and no manager gate.

- `boarding_feeding_instruction_conflict_routes_to_staff_review_not_default_meal_plan`
  - Conflicting profile/provider/customer feeding sources produce `ConflictingInstruction` and never a guessed default.

- `boarding_medication_absent_produces_no_medication_tasks`
  - Empty `CareProfile.medications` returns `MedicationReview::NoMedication` and no medication administration task.

- `boarding_medication_requires_review_blocks_checkin_until_staff_or_manager_approval`
  - `MedicationReviewRequirement::RequiresReview` produces a medication review gate and keeps readiness blocked or conditionally ready according to policy.

- `boarding_medication_missing_schedule_routes_to_vet_clarification`
  - An imported/vague medication note without executable schedule/dose cannot produce `AdministerDose`; it creates `VetClarificationRequired` or `ManagerReviewRequired`.

- `boarding_medication_change_during_active_stay_invalidates_future_administration_tasks`
  - A profile-change event creates a new instruction snapshot and replaces or blocks future tasks derived from the prior snapshot.

- `boarding_behavior_bite_history_requires_manager_review_and_blocks_group_play_recommendation`
  - `temperament::BehaviorObservation::BiteHistory` maps to behavior review, playtime ineligibility/review, and manager-sensitive communication boundary.

- `boarding_behavior_anxiety_flag_creates_special_handling_handoff_without_customer_message_send`
  - Anxiety/special-handling facts create internal handoff items and draft-only customer text.

- `boarding_cat_behavior_plan_uses_individual_enrichment_not_dog_group_play_assumptions`
  - Cat boarding maps behavior/playtime decisions to cat individual enrichment; dog group-play rules do not leak across species.

- `boarding_care_plan_ready_for_checkin_requires_all_blocking_gates_closed`
  - `care::Plan::readiness()` cannot be `ReadyForCheckIn` while any blocking gate remains open.

- `boarding_handoff_packet_includes_open_feeding_medication_behavior_and_exception_tasks`
  - Handoff packet carries open care obligations with typed reasons, not free-text-only summaries.

- `boarding_staff_task_mapping_preserves_care_gate_reason_and_source_evidence`
  - Mapping into `operations::StaffTask` preserves Boarding task kind, gate, source, and affected pet/reservation.

- `boarding_pawgress_report_with_medication_or_behavior_content_requires_review_before_send`
  - Sensitive content sets `report::ApprovalState::ManagerReviewRequired` or customer-message review; draft cannot be sent.

- `boarding_agent_can_draft_care_tasks_but_cannot_mark_medication_administered`
  - Approval policy permits internal drafts and denies/blocks completion of medication administration by AI.

- `boarding_care_audit_records_review_gate_open_and_close_without_leaking_raw_note_text`
  - Audit trail records typed source/decision/actor while preserving redacted note values.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Expand `operations::boarding` with child modules or nested types for `care`, `care::feeding`, `care::medication`, `care::behavior`, `task`, `handoff`, and `agent` as behavior grows.
  - Preserve `operations::boarding::Contract` as the service-line root; avoid creating a parallel `BoardingCareContract` disconnected from it.

- `domain/src/care.rs`
  - Reuse existing redacted `FeedingInstruction`, `MedicationName`, `MedicationDose`, `MedicationSchedule`, `ReviewReason`, `MedicationReviewRequirement`.
  - Add only master-care-profile concepts here; stay-scoped Boarding decisions belong under `operations::boarding::care`.

- `domain/src/temperament.rs`
  - Reuse `BehaviorObservation`, `GroupPlayObservation`, `TemperamentRating`, `StaffNote`.
  - Add new behavior observations only if they are stable cross-service temperament facts. Boarding-only fulfillment interpretations belong in `boarding::care::behavior`.

- `domain/src/entities.rs`
  - `CareProfile`, `MedicationInstruction`, `Reservation`, `HardStop`, and `Pet` will be consumed. Avoid putting Boarding stay readiness on entities unless it is truly cross-service.

- `domain/src/policy.rs`
  - Reuse `PlayEligibilityPolicy`, `ReviewGate`, `AutomationLevel`, and policy denial reasons.
  - Add broad policy gates only when they apply outside Boarding; otherwise keep gates Boarding-local and map outward.

- `domain/src/workflow.rs`, `domain/src/tools.rs`, `domain/src/agents.rs`
  - Add workflow/tool/agent surfaces only as ports or specs. Do not let tools decide care readiness.

- `domain/tests/*`
  - Add semantic tests named above. Existing tests in `domain/tests/domain_quality_patterns.rs` and `domain/tests/petsuites_core_service_contracts.rs` show the preferred style: typed domain values, redacted care values, and semantic test names.

- `docs/domain/petsuites/boarding/service-domain-map.md`
  - This implication refines the care fulfillment section and should remain consistent with its acceptance-test list.

### Migration and refactor risks

- Current `operations::BoardingCareFeature::{FeedingSupport, MedicationSupport}` is catalog vocabulary, not an executable care plan. Do not overload it with review outcomes.
- Current `operations::StaffTaskKind::{Feeding, MedicationAdministration, PlaygroupAssessment, IncidentFollowUp}` is useful but too generic to preserve Boarding care gate/source details by itself. Use Boarding-owned task drafts and map into generic tasks at the boundary.
- Existing `entities::CareProfile.feeding_instructions: Option<care::FeedingInstruction>` makes missing instructions explicit; keep that semantic pressure instead of defaulting to a synthetic feeding instruction.
- `MedicationReviewRequirement::RequiresReview` is already a semantic hook. Later cards should honor it when building `boarding::care::MedicationReview` rather than inventing a parallel boolean.
- `temperament::BehaviorObservation::indicates_behavior_review_evidence()` is intentionally conservative. Boarding behavior policy may add accommodation/play/report-specific interpretation, but should not weaken the cross-service safety signal.
- Storage/provider imports may contain raw notes. Convert into redacted domain values plus source provenance before review; do not serialize raw provider note blobs into domain audit logs.
- Pawgress Report generation depends on care review. The report card should not be implemented before the sensitive-content review state exists, or it will create unsafe customer-message paths.

### Dependencies on other Boarding implications

- Capacity/accommodation: care readiness may depend on accommodation species/type, quiet-room/special-handling availability, and room assignment feasibility.
- Stay/check-in/check-out flow: check-in cannot complete until blocking care gates close; checkout/Pawgress cannot complete while care exceptions are unresolved.
- Playtime/add-ons/upsells: behavior and medication/feeding constraints suppress unsafe playtime, exit bath, training, grooming, or premium-report recommendations.
- Pawgress Report/customer communication: medication/behavior/incident facts require approval-state integration and audit-backed source evidence.
- Staff shift handoff: feeding, medication, behavior, and exception tasks must be included in handoff packets until resolved.
- Payment/deposit/cancellation: care review should not mutate payment or reservation state directly; it may create recommended actions or hard stops that other policies consume.

### Recommended implementation slice

1. Add `operations::boarding::care` review enums and `care::Plan` without storage first.
2. Add policy tests for missing feeding, medication review, and behavior/play blocks.
3. Add task draft mapping into `operations::StaffTask` while preserving Boarding source/gate metadata.
4. Add audit/source provenance types and storage codecs.
5. Add agent approval-policy tests for care summarization, task drafting, report drafting, and denied completion/send actions.
6. Only then wire Pawgress/customer-message generation to care evidence and report sensitivity.
