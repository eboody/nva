# Daycare implication 02: group assignment

Purpose: define the operational and domain contract for assigning a daycare guest to a safe care lane or playgroup. This implication narrows the service-map concept of `Playgroup assignment` into the types, policies, repository contracts, review gates, and tests that later Rust cards should implement.

Assumption: when local PetSuites policy details are unknown, the safest extensible model is to treat automation as read/draft/recommend only, require staff confirmation for behavior-based assignments, and require manager approval for any override of eligibility, capacity, ratio, or incident restrictions.

## 1. Operational story

### Trigger

Group assignment is triggered when one of these events occurs:

- A dog daycare reservation/check-in requests `operations::daycare::ServiceVariant::AllDayPlay` or `HalfDayPlay`.
- A `DayPlayPlusRoom` reservation needs the play/enrichment lane chosen in addition to the room/rest lane.
- A staff member changes a pet's temperament, incident, vaccine, spay/neuter, care-note, or restriction evidence in a way that invalidates the current assignment.
- A manager/front-desk preparation workflow needs the next operating day's daycare roster reviewed before check-in.
- A same-day capacity or staffing change requires moving pets between group-play lanes, individual care, waitlist, or manager review.

### Actors

- Pet guest: the dog or cat receiving daycare service. Group play applies only to dog group-play variants.
- Customer/owner: receives only staff-approved customer-safe messages; never receives raw internal notes directly from assignment automation.
- Front desk: gathers missing requirements, explains unresolved readiness states, and creates/reviews customer follow-up drafts.
- Daycare/kennel staff: observes temperament, assigns or confirms playgroup fit, records group changes and incidents.
- Lead staff/manager: approves overrides, reinstatement after incidents, capacity/ratio exceptions, and sensitive customer communications.
- Automation/agent workflows: classify requests, collect typed evidence, propose assignments/tasks, summarize risks, and draft messages behind review gates.
- Source systems/adapters: reservation system, customer/pet profile store, vaccine/document source, staff schedule/capacity source, and incident/care-note records.

### Inputs

Core inputs should be promoted into semantic values before assignment behavior branches on them:

- `operations::daycare::ReservationRequest`: customer, pet, location, service variant, requested date/window, source reservation, and optional package/membership reference.
- `operations::daycare::GroupPlayEligibilityDecision`: current eligibility outcome from typed evidence, not a raw `eligible: bool`.
- `operations::daycare::EligibilityEvidence`: pet identity/species/age, temperament evidence, vaccine status, spay/neuter status when group play is possible, incident restrictions, care-note review state, and staff/capacity snapshot.
- `operations::daycare::RosterSnapshot`: current playgroups/care lanes, roster limits, current assignments, scheduled staff, and location contract ratio.
- `operations::daycare::StaffCoverageDecision`: sufficient/insufficient/unknown/override-required coverage for the proposed roster.
- `care::*` review requirements for feeding, medication, allergies, medical notes, and handling instructions.
- `temperament::*` observations and staff notes as evidence, not as final assignment decisions.
- `policy::ReviewGate` and location-specific `operations::daycare::Contract` values.

### Decisions

The assignment workflow makes these deterministic decisions before any staff-facing output is considered ready:

1. Does the requested `ServiceVariant` require dog group play, individual dog care, hybrid play-and-room care, or cat individual enrichment?
2. Is the pet currently eligible for the target care mode, or does the evidence produce `NeedsStaffReview`, `Ineligible`, or `TemporarilySuspended`?
3. If group play is possible, which playgroup lane best matches size, age/life stage, energy level, temperament, known behavior flags, and local grouping rules?
4. Does the target lane have roster capacity and staff coverage under `StaffCoveragePolicy`?
5. Do medical/care notes, allergies, medication timing, or handling instructions require individual care, staff review, or manager approval before group placement?
6. Is this an initial assignment, a same-day reassignment, a temporary split/rest period, or a post-incident restricted assignment?
7. What staff tasks, workflow events, audit records, and customer-safe draft messages should be produced?

Unknown, missing, stale, or conflicting evidence must never default to group-play eligibility. It routes to typed review or individual/non-group care when policy permits.

### Outputs

- `operations::daycare::PlaygroupAssignment` for a concrete group-play placement.
- `operations::daycare::CareLaneAssignment` for individual day boarding, cat enrichment, room/rest lane, review lane, or waitlist.
- `operations::daycare::AssignmentDecision`, with explicit success/review/denial/reassignment variants and typed reasons.
- `operations::daycare::AssignmentAuditRecord` describing inputs, policy version, actor/agent, decision, rationale, review gate, and resulting tasks/events.
- Staff tasks such as `operations::StaffTaskKind::PlaygroupAssessment`, `DocumentReview`, `IncidentFollowUp`, `CheckInPrep`, or `CustomerFollowUp`.
- Workflow events for reservation readiness, assignment proposed/confirmed/review-required, and eligibility invalidated.
- Agent outputs: manager daily brief risk rows, front-desk readiness summaries, and customer-message drafts that require approval before sending.

### Success state

A successful group assignment means:

- The requested care mode is compatible with species, service variant, pet profile, and policy.
- The pet has an `Eligible` group-play decision for dog group play, or an explicit non-group care decision for day boarding/cat/individual care.
- The target group or care lane has capacity and sufficient staff coverage.
- The assignment rationale is semantic and auditable, not a raw note blob.
- Any review gates are either satisfied by the correct staff/manager actor or remain open with explicit tasks.
- No customer-facing promise or message is sent automatically.

### Failure and exception states

- Missing or stale temperament evidence: `NeedsStaffReview { reason: MissingTemperamentAssessment | StaleTemperamentAssessment }`; create `PlaygroupAssessment`.
- Unknown vaccine/spay-neuter/care facts: review state, document/care-note task, no group-play assignment.
- Ineligible for group play but safe for individual care: route to `DogIndividualDayBoarding` if capacity and care requirements allow.
- Suspended or unresolved incident: `TemporarilySuspended`; manager review required before any group-play reinstatement.
- Insufficient staff or capacity: no assignment confirmation; produce capacity/ratio risk and manager/front-desk task.
- Hybrid `DayPlayPlusRoom` conflict: can only succeed when both play/enrichment lane and room/rest lane are available; otherwise review/waitlist.
- Cat individual playtime: never forced through dog playgroup matching; use cat enrichment/room handling semantics.
- Same-day staff override: allowed only as an auditable human decision with role, reason, timestamp, and policy exception scope.

## 2. Domain types to add or refine

Keep daycare-specific terms under `operations::daycare` so call sites preserve meaning.

### Core assignment values

```rust
operations::daycare::AssignmentDecision::{
    AssignedToPlaygroup { assignment: PlaygroupAssignment },
    AssignedToCareLane { assignment: CareLaneAssignment },
    NeedsStaffReview { reasons: Vec<AssignmentReviewReason>, tasks: Vec<StaffTaskDraft> },
    Waitlisted { reason: AssignmentWaitlistReason },
    Denied { reasons: Vec<AssignmentDenialReason> },
}

operations::daycare::PlaygroupAssignment {
    assignment_id: AssignmentId,
    pet_id: entities::PetId,
    reservation_id: entities::ReservationId,
    location_id: entities::LocationId,
    playgroup_id: PlaygroupId,
    group_profile: PlaygroupProfile,
    scheduled_window: AttendanceWindow,
    staff_coverage: StaffCoverageDecision,
    rationale: AssignmentRationale,
    review_state: AssignmentReviewState,
}

operations::daycare::CareLaneAssignment {
    assignment_id: AssignmentId,
    pet_id: entities::PetId,
    reservation_id: entities::ReservationId,
    location_id: entities::LocationId,
    care_mode: CareMode,
    lane_id: CareLaneId,
    scheduled_window: AttendanceWindow,
    staff_coverage: StaffCoverageDecision,
    rationale: AssignmentRationale,
    review_state: AssignmentReviewState,
}
```

### Newtypes and invariants

- `AssignmentId`: non-empty provider-neutral assignment identifier; external roster IDs convert at storage/adapter boundaries.
- `PlaygroupId`: non-empty provider-neutral group identifier. Already proposed by the service map; should not be a `String` at behavior call sites.
- `CareLaneId`: non-empty provider-neutral lane/room/rest/enrichment identifier for non-group care.
- `RosterSnapshotId`: non-empty snapshot identifier tying an assignment to capacity/staff facts used at decision time.
- `RosterLimit`: non-zero upper bound for a group/care lane.
- `AssignmentRationale`: non-empty redacted text/value list; `Debug` should not leak raw staff notes if it may contain sensitive behavior detail.
- `GroupLabel`: non-empty local display name such as “small calm dogs”; display-only, not a policy key.
- `AssignmentSequence`: positive ordinal when source systems require ordered roster lanes.

### Enums and explicit invariants

```rust
operations::daycare::CareMode::{
    DogGroupPlay,
    DogIndividualDayBoarding,
    DogHybridPlayAndRoom,
    CatIndividualEnrichment,
    StaffReviewHolding,
    CapacityWaitlist,
}

operations::daycare::PlaygroupProfile {
    size_band: DogSizeBand,
    energy_band: EnergyBand,
    life_stage: LifeStageBand,
    temperament_band: TemperamentBand,
}

operations::daycare::DogSizeBand::{Small, Medium, Large, Giant, UnknownRequiresReview}
operations::daycare::EnergyBand::{Low, Moderate, High, UnknownRequiresReview}
operations::daycare::LifeStageBand::{Puppy, Adult, Senior, UnknownRequiresReview}
operations::daycare::TemperamentBand::{IntroOnly, Easygoing, Structured, ManagerReviewRequired}

operations::daycare::AssignmentReviewReason::{
    MissingTemperamentAssessment,
    StaleTemperamentAssessment,
    UnknownSizeOrEnergyFit,
    CareNoteRequiresReview,
    IncidentPendingReview,
    StaffCoverageUnknown,
    CapacityUncertain,
    ManagerOverrideRequested,
}

operations::daycare::AssignmentDenialReason::{
    GroupPlayIneligible { reason: EligibilityDenialReason },
    TemporarilySuspended { incident_id: IncidentId },
    StaffCoverageInsufficient,
    PlaygroupCapacityUnavailable,
    ServiceUnavailableForSpecies,
    CareModeUnavailableAtLocation,
}

operations::daycare::AssignmentReviewState::{
    AutomationProposed,
    StaffConfirmed { staff_id: entities::StaffId },
    ManagerApprovedOverride { manager_id: entities::StaffId, reason: OverrideReason },
    Rejected { staff_id: entities::StaffId, reason: AssignmentDenialReason },
}
```

Invariants:

- `PlaygroupAssignment` can only be constructed from `GroupPlayEligibilityDecision::Eligible` and `StaffCoverageDecision::Sufficient` unless the review state is an explicit manager-approved override.
- `CareMode::DogGroupPlay` is dog-only and never valid for `CatIndividualPlaytime`.
- `CareMode::DogIndividualDayBoarding` is a separate safe operational lane, not a failed group assignment disguised as success.
- `CareMode::DogHybridPlayAndRoom` must carry both play/enrichment capacity and room/rest capacity checks.
- Unknown size/energy/temperament bands are review reasons, not silent matches.
- Capacity and ratio values use non-zero semantic counts; zero cannot be represented as a valid roster limit or ratio denominator.

### Types to refine from current code

- Split current `operations::daycare::GroupAssignmentRule` into:
  - `GroupAssignmentPolicy` or `GroupMatchingPolicy`: the local policy/rule set.
  - `PlaygroupAssignment` and `CareLaneAssignment`: concrete decisions.
- Move/re-export top-level `operations::DaycareFormat` into `operations::daycare::ServiceVariant`.
- Promote `operations::daycare::EligibilityRequirement` into decision-making policy inputs and evidence requirements rather than a passive list.
- Extend `StaffPetRatio` with `StaffCoveragePolicy` and `RosterSnapshot` so ratio math is owned by daycare coverage policy, not call-site helpers.
- Keep `policy::ConservativePlayEligibilityPolicy` as a neighboring coarse policy, but implement daycare-specific `GroupPlayEligibilityPolicy` that returns richer review/denial/suspension outcomes.

## 3. Relationship map between types

### Entities

- `entities::PetId`: target of eligibility and assignment. Assignment is per pet/reservation/date, not a permanent property of the pet.
- `entities::CustomerId`: owner context for follow-up drafts and package opportunities; not the owner of assignment behavior.
- `entities::ReservationId`: source reservation/check-in request that assignment prepares or updates.
- `entities::LocationId`: scopes local policy, rosters, capacity, staff ratio, group labels, and manager overrides.
- `entities::StaffId`: reviewer/approver identity for staff-confirmed and manager-approved assignment states.

### Value objects

- `ServiceVariant`, `CareMode`, `AttendanceWindow`, `PlaygroupId`, `CareLaneId`, `RosterLimit`, `StaffPetRatio`, `StaffCount`, `PetCount`, `AssignmentRationale`, `PlaygroupProfile`, `RosterSnapshotId`.
- `temperament::GroupPlayObservation`, `TemperamentRating`, `BehaviorObservation`, and redacted `StaffNote` feed evidence, not final assignment state.
- `care::*` review requirements gate safe handling and may alter care lane or review state.

### Policies

- `GroupPlayEligibilityPolicy`: decides whether group play can be considered.
- `GroupAssignmentPolicy`: chooses compatible playgroup/care lane candidates from eligibility, pet profile, temperament evidence, and roster state.
- `StaffCoveragePolicy`: evaluates whether the resulting roster satisfies ratio/staffing invariants.
- `IncidentPolicy`: invalidates or restricts assignments after safety/behavior/health events.
- `FrontDeskThroughputPolicy`: decides whether check-in is ready or needs tasks/review.

### Repositories and stores

- `PolicyRepository`: loads `Contract`, local matching policy, ratio, eligibility, incident, vaccine/spay-neuter, and override rules by location/date.
- `RosterRepository`: loads/saves roster snapshots, playgroup/care lane capacity, current assignments, and staff schedule references.
- `EligibilityRepository`: stores eligibility decisions/evidence snapshots and invalidates them when upstream facts change.
- `AssignmentRepository`: persists assignment decisions, audit records, staff confirmations, reassignments, and manager overrides.
- `IncidentRepository`: reads unresolved incident restrictions and appends incident follow-up disposition links.

### Workflow events

- `DaycareAssignmentRequested`
- `DaycareAssignmentProposed`
- `DaycareAssignmentConfirmed`
- `DaycareAssignmentReviewRequired`
- `DaycareAssignmentDenied`
- `DaycareAssignmentInvalidated`
- `DaycareAssignmentManagerOverrideApproved`

These may map to existing `workflow::WorkflowEvent`/`WorkflowResult` shapes first, then become typed daycare events once the module grows.

### Staff tasks

- `StaffTaskKind::PlaygroupAssessment { pet_id }`: temperament/group-fit review.
- `StaffTaskKind::DocumentReview { pet_id }`: vaccine, spay/neuter, or other document ambiguity.
- `StaffTaskKind::IncidentFollowUp { pet_id }`: unresolved safety/behavior/health incident review.
- `StaffTaskKind::CheckInPrep { reservation_id }`: front-desk readiness and day-of assignment prep.
- `StaffTaskKind::CustomerFollowUp { customer_id, reason }`: customer-safe request for missing information or schedule changes.

Later code may introduce daycare-specific task payloads instead of overloading generic task kinds, but the first implementation can use these existing semantic variants.

### Agent specs and tools

- `booking-triage`: reads policy/availability/eligibility context, proposes assignment readiness, cannot confirm booking or override hard policy.
- `manager-daily-brief`: surfaces ratio/capacity risks, unresolved assignment reviews, incident restrictions, and coverage warnings.
- `daily-care-update`: drafts owner-safe updates after staff-approved notes/photos; does not disclose raw internal notes automatically.
- `incident-escalation`: summarizes incident facts and manager/customer review packets; cannot close incidents or reinstate eligibility.
- Tool contracts: `availability-read`, `policy-read`, `reservation-read`, `care-note-read`, `document-read`, `task-create`, and `draft-message`. No tool should auto-send owner-facing assignment/incident messages.

## 4. Interaction contract

Rust-like pseudo-signatures below name ownership. Behavior belongs on daycare policies, repositories, services, or domain values; avoid free-floating helper functions.

```rust
pub trait operations::daycare::PolicyRepository {
    async fn contract_for(
        &self,
        location: entities::LocationId,
        day: OperatingDay,
    ) -> operations::daycare::Result<operations::daycare::Contract>;

    async fn group_assignment_policy_for(
        &self,
        location: entities::LocationId,
        day: OperatingDay,
    ) -> operations::daycare::Result<operations::daycare::GroupAssignmentPolicy>;
}

pub trait operations::daycare::RosterRepository {
    async fn snapshot_for(
        &self,
        location: entities::LocationId,
        day: OperatingDay,
    ) -> operations::daycare::Result<operations::daycare::RosterSnapshot>;

    async fn persist_assignment(
        &self,
        decision: operations::daycare::AssignmentDecision,
        audit: operations::daycare::AssignmentAuditRecord,
    ) -> operations::daycare::Result<()>;
}

pub trait operations::daycare::AssignmentRepository {
    async fn current_for_reservation(
        &self,
        reservation: entities::ReservationId,
    ) -> operations::daycare::Result<Option<operations::daycare::AssignmentDecision>>;

    async fn append_review(
        &self,
        assignment: operations::daycare::AssignmentId,
        review: operations::daycare::AssignmentReviewState,
    ) -> operations::daycare::Result<()>;
}
```

```rust
impl operations::daycare::GroupPlayEligibilityPolicy {
    pub fn evaluate(
        &self,
        evidence: operations::daycare::EligibilityEvidence,
    ) -> operations::daycare::GroupPlayEligibilityDecision;
}

impl operations::daycare::StaffCoveragePolicy {
    pub fn evaluate(
        &self,
        roster: &operations::daycare::RosterSnapshot,
        candidate: &operations::daycare::AssignmentCandidate,
    ) -> operations::daycare::StaffCoverageDecision;
}

impl operations::daycare::GroupAssignmentPolicy {
    pub fn candidate_for(
        &self,
        request: &operations::daycare::ReservationRequest,
        eligibility: &operations::daycare::GroupPlayEligibilityDecision,
        roster: &operations::daycare::RosterSnapshot,
    ) -> operations::daycare::AssignmentCandidateDecision;
}
```

```rust
impl operations::daycare::AssignmentService {
    pub async fn assign(
        &self,
        request: operations::daycare::ReservationRequest,
    ) -> operations::daycare::Result<operations::daycare::AssignmentDecision>;

    pub async fn confirm_staff_review(
        &self,
        assignment: operations::daycare::AssignmentId,
        staff: entities::StaffId,
        outcome: operations::daycare::StaffAssignmentReviewOutcome,
    ) -> operations::daycare::Result<operations::daycare::AssignmentDecision>;

    pub async fn approve_manager_override(
        &self,
        assignment: operations::daycare::AssignmentId,
        manager: entities::StaffId,
        override_reason: operations::daycare::OverrideReason,
    ) -> operations::daycare::Result<operations::daycare::AssignmentDecision>;

    pub async fn invalidate_for_event(
        &self,
        event: operations::daycare::AssignmentInvalidationEvent,
    ) -> operations::daycare::Result<Vec<operations::daycare::AssignmentDecision>>;
}
```

Service behavior:

1. Load location/day `Contract`, group assignment policy, roster snapshot, eligibility evidence, incident restrictions, and care-note review state.
2. Evaluate `GroupPlayEligibilityPolicy` before constructing any `PlaygroupAssignment`.
3. Ask `GroupAssignmentPolicy` for a candidate care lane/playgroup; service-variant semantics decide whether group play is required, optional, or forbidden.
4. Evaluate `StaffCoveragePolicy` against the candidate roster.
5. Return a typed `AssignmentDecision` and persist an `AssignmentAuditRecord` in the same application transaction when storage supports it.
6. Create staff tasks for review states. Do not mutate reservations, send customer messages, change staff schedules, or override restrictions as a side effect of assignment recommendation.

## 5. Review and approval contract

### Automation level

- Safe to automate: read source facts, normalize them into evidence, calculate deterministic policy outcomes, propose assignment candidates, draft internal tasks, and create manager/front-desk brief rows.
- Draft/internal-task only: customer follow-up text, incident/customer messaging, schedule/reservation update drafts, and package/opportunity suggestions.
- Never automate without approval: final behavior assessment, manager override, incident suspension/reinstatement, customer-facing safety/health/incident message send, payment/discount/package action, and staff schedule changes.

### Review gates

- `policy::ReviewGate::BehaviorReview`: required for initial temperament assessment, stale/unknown temperament evidence, behavior flags, group-fit uncertainty, and staff rejection of proposed assignment.
- `policy::ReviewGate::MedicalDocumentReview`: required for ambiguous vaccine, spay/neuter, medical, allergy, or medication evidence.
- `policy::ReviewGate::ManagerApproval`: required for overriding group-play ineligibility, staff ratio/capacity limits, incident restrictions, or local policy.
- `policy::ReviewGate::CustomerMessageApproval`: required before any owner-facing assignment, requirement, health, behavior, or incident message is sent.

### Audit trail

Every proposed, confirmed, denied, invalidated, or overridden assignment should record:

- assignment id, reservation id, pet id, customer id if available, location id, operating day/window;
- service variant, target care mode, target playgroup/lane;
- eligibility decision and evidence snapshot id;
- roster snapshot id, ratio/coverage decision, and capacity state;
- policy id/version or contract snapshot;
- actor type (`Agent`, `Staff`, `Manager`, `SystemInvalidation`) and actor id/tool id where available;
- review gates required/satisfied;
- rationale and typed reasons without leaking unredacted staff notes to customer-safe surfaces.

### Customer/member-facing boundaries

- A proposed assignment is internal operational state. It is not a promise of availability or confirmed service until reservation/availability policies also allow confirmation.
- Customer messages may say that staff will review or that requirements are needed; they must not expose raw internal behavior labels, sensitive health speculation, or unapproved incident conclusions.
- Automation can draft “we need updated vaccine proof” or “our team will complete a playgroup assessment,” but cannot send it without the configured approval gate.
- Automation must not hide concerning behavior, health, or incident facts from staff/manager review packets.

## 6. Test contracts

Named semantic tests for later implementation:

1. `all_day_play_assignment_requires_eligible_group_play_decision`
   - A full-day play request with `NeedsStaffReview` eligibility cannot produce `AssignmentDecision::AssignedToPlaygroup`.

2. `half_day_play_uses_same_group_assignment_safety_gates_as_full_day_play`
   - Half-day play differs by attendance window/pricing only; it still requires temperament, eligibility, coverage, and capacity gates.

3. `day_boarding_routes_group_play_denied_dog_to_individual_care_when_safe`
   - A dog denied for group play can receive `CareMode::DogIndividualDayBoarding` only when individual-care capacity and care-note checks pass.

4. `cat_individual_playtime_never_constructs_dog_playgroup_assignment`
   - `ServiceVariant::CatIndividualPlaytime` returns cat enrichment/individual care or review, never `CareMode::DogGroupPlay`.

5. `day_play_plus_room_requires_both_room_and_play_lane_capacity`
   - Hybrid assignment fails/reviews/waitlists if either play lane or room/rest lane lacks capacity/coverage.

6. `unknown_temperament_size_or_energy_routes_to_staff_review`
   - Unknown matching bands produce `AssignmentReviewReason`, not a default playgroup.

7. `suspending_incident_invalidates_current_playgroup_assignment`
   - An unresolved suspending incident changes an existing playgroup assignment into review/suspended state and creates incident follow-up.

8. `insufficient_staff_coverage_blocks_assignment_confirmation`
   - `StaffCoverageDecision::Insufficient` prevents staff-unreviewed group assignment and surfaces capacity/labor risk.

9. `manager_override_records_audit_and_does_not_erase_original_denial`
   - Override preserves original denial/review reason, manager id, timestamp, and override rationale.

10. `staff_rejection_of_agent_candidate_creates_reviewed_assignment_state`
    - Staff can reject an automation-proposed group with typed reason; the audit trail records both proposed and rejected states.

11. `assignment_rationale_debug_redacts_sensitive_staff_notes`
    - Rationale values containing staff-note-derived text do not leak raw note contents in `Debug` output.

12. `roster_snapshot_id_ties_assignment_to_capacity_facts`
    - Persisted assignment references the roster snapshot used for ratio/capacity evaluation.

13. `customer_message_draft_for_missing_assignment_requirement_requires_approval`
    - Missing requirement follow-up is a draft/task with `CustomerMessageApproval`, not an auto-send.

14. `service_variant_paths_preserve_daycare_meaning_in_assignment_api`
    - Public examples use `operations::daycare::ServiceVariant`, `GroupAssignmentPolicy`, and `PlaygroupAssignment` rather than raw strings or top-level daycare booleans.

15. `assignment_repository_roundtrips_semantic_decisions_without_raw_string_branching`
    - Storage adapters preserve enum/newtype decisions and reject invalid/unknown required values rather than defaulting silently.

## 7. Integration notes for the serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Current home of `operations::daycare`; likely first target for new assignment contracts unless daycare is split into `domain/src/operations/daycare.rs`.
- `domain/src/policy.rs`
  - Existing `PlayEligibilityPolicy`, `ReviewGate`, and conservative play eligibility values may be reused/refined or bridged into richer daycare-specific decisions.
- `domain/src/temperament.rs`
  - Existing temperament observations feed assignment evidence. Avoid moving final assignment decisions into temperament.
- `domain/src/care.rs`
  - Care-note review requirements should gate care lane/group assignment where safety or handling instructions matter.
- `domain/src/workflow.rs`
  - Typed workflow events may be added or mapped from existing event/result types.
- `domain/src/agents.rs`
  - Baseline agent specs may need daycare assignment wording/tool boundaries for booking triage and manager daily brief.
- `domain/src/tools.rs`
  - Tool contracts may need semantic daycare assignment/roster requests rather than generic service notes.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Extend contract tests with assignment-specific invariants.
- `domain/tests/domain_quality_patterns.rs`
  - Add quality tests ensuring no raw strings/booleans replace semantic assignment paths.

### Migration/refactor risks

- `GroupAssignmentRule` currently mixes policy/rule language with concrete operational outcome. Split carefully to avoid breaking the existing `daycare_contract_encodes_attendance_packages_ratios_groups_incidents_and_eligibility` test.
- Top-level `operations::DaycareFormat` and `DaycareEligibilityRule` currently preserve service offering compatibility. Re-export or adapter-map them to `operations::daycare::ServiceVariant`/`EligibilityRequirement` before deleting old names.
- `policy::ConservativePlayEligibilityPolicy` returns a binary-ish eligible/ineligible shape. Do not stretch it into assignment ownership; add daycare-specific policy decisions and bridge coarse policy output as evidence or compatibility.
- Staff ratio math can become scattered if implemented as helper functions. Keep it on `StaffCoveragePolicy`, `RosterSnapshot`, or `StaffPetRatio` methods.
- Storage adapters may receive provider-specific group names/codes. Convert to `PlaygroupId`, `CareLaneId`, `GroupLabel`, and semantic enums at the boundary.
- Staff notes and behavior labels may contain sensitive details. Use redacted debug/display patterns similar to `temperament::StaffNote`.

### Dependencies on other implications

- Depends on the daycare service-domain map's eligibility, staff ratio, incident, front-desk readiness, and daily attendance concepts.
- Upstream dependency: group-play eligibility/evidence modeling should exist before concrete `PlaygroupAssignment` constructors become strict.
- Upstream dependency: roster/capacity snapshots and staff coverage policy should exist before assignment can confirm rather than propose.
- Downstream dependency: front-desk readiness can consume assignment decisions to decide check-in status.
- Downstream dependency: incident handling must invalidate assignments and govern suspension/reinstatement.
- Downstream dependency: daily recurring attendance can materialize repeated assignment requests but should not silently confirm them.
- Downstream dependency: package/membership opportunity logic can read attendance/assignment history but must not change assignment safety gates.

Doc-only status: this artifact models the operational implication. It does not change code, storage schemas, live systems, member-facing data, or operational policy.
