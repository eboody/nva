# Daycare operational implication 03: Staff-to-pet ratios

Purpose: model staff-to-pet ratios as a first-class daycare safety and capacity contract. The ratio is not a naked integer used by scheduling helpers; it is a location-scoped operational policy that determines whether a daycare roster can be offered, checked in, assigned to a playgroup, or escalated for manager review.

Scope assumptions:

- Default PetSuites daycare ratio is represented by the existing `operations::daycare::StaffPetRatio::new(StaffCount(1), PetCount(12))` shape, but the real threshold can vary by location, care mode, playgroup, room, time block, incident posture, and manager override policy.
- Ratio policy protects supervised care. It does not itself decide temperament, vaccines, spay/neuter status, room capacity, payment, or customer messaging; it consumes those adjacent facts only where they change the roster that must be supervised.
- Unknown or stale staff/roster evidence must route to review or waitlist, never to automatic group-play eligibility.
- Modeling only: this document does not change live staffing, reservations, pricing, customer messages, or member-facing data.

## 1. Operational story

Trigger:

- A daycare reservation request, recurring attendance materialization, same-day check-in, roster reassignment, staff schedule change, incident, late arrival/departure, or manager daily-brief build needs to know whether a daycare care lane has enough qualified staff for the pets assigned to it.
- The trigger can originate from `booking-triage`, front desk check-in, manager daily brief, a staff schedule review, or an incident escalation workflow.

Actors:

- Front desk staff: need a ready/waitlist/review decision before promising availability or checking a pet in.
- Daycare lead / kennel technician: owns playgroup and individual-care lane rosters during the operating day.
- Manager: approves ratio exceptions, staff schedule changes, waitlist release, and any override that affects safety or labor.
- Customer/member: receives only approved availability or follow-up language; ratio details are usually internal.
- AI agents: may read snapshots, detect risk, draft internal tasks/messages, and summarize ratio state, but may not change staffing or confirm availability autonomously.

Inputs:

- `entities::LocationId` and operating day/time block.
- Location daycare contract from `operations::daycare::PolicyRepository`, including `StaffPetRatio`, care-mode-specific overrides, allowed override policy, and review gates.
- Current and planned daycare roster from `operations::daycare::RosterRepository`: pet ids, reservation ids, care mode, playgroup/room/lane assignment, waitlist candidates, late arrival/departure risks, and capacity limits.
- Scheduled staff snapshot: staff ids, role/qualification, shift time windows, break/absence flags, assigned lane(s), and whether a staff member can count toward group-play supervision.
- Eligibility/care facts that alter the roster: group-play eligibility decision, incident restrictions, medical/care review requirements, and room/rest requirements for `DayPlayPlusRoom` or individual care.

Decisions:

- Which ratio policy applies: location default, dog group-play, dog individual day boarding, hybrid play-and-room, cat individual enrichment, senior/low-energy group, incident-heightened supervision, or manager-approved exception.
- Which pets count in which care lane and time block. A dog in individual day boarding must not inflate a dog group-play roster; a hybrid pet can create both play/enrichment and room/rest coverage requirements.
- Which scheduled staff count toward supervision. Qualification, assigned role, overlapping shift window, breaks, and staff pulled into incident/document/customer tasks matter.
- Whether coverage is `Sufficient`, `Insufficient`, `Unknown`, or `ManagerOverrideRequired`.
- Whether the downstream operational state is ready to check in/assign, waitlist, suggest staff schedule review, escalate to manager, or block group-play assignment.

Outputs:

- `operations::daycare::StaffCoverageDecision` with typed reasons and a ratio calculation trace.
- `operations::daycare::StaffCoverageSnapshot` for audit and manager daily brief summaries.
- Optional `operations::StaffTaskKind::CheckInPrep`, `PlaygroupAssessment`, `IncidentFollowUp`, or `CustomerFollowUp` task recommendations when coverage is unresolved or customer follow-up is needed.
- `operations::OperationsRisk::LaborMismatch` or `OperationsRisk::PetSafetyOrCareRisk` for manager attention.
- `operations::OperationsAction::SuggestScheduleReview` or `EscalateToManager` for manager-approved actions.
- A customer-message draft only when staff choose to communicate waitlist/unavailability; the draft carries a review gate and must not expose internal staffing details unnecessarily.

Success state:

- Every daycare pet assigned for the time block has exactly one truthful care lane, each lane has a current ratio policy, sufficient qualified scheduled staff, and a persisted/auditable coverage decision.
- Group-play assignment can proceed only when group-play eligibility and staff coverage are both satisfied.
- Front desk readiness can say `Ready` only when ratio/capacity/eligibility/care/payment requirements are all resolved.

Failure and exception states:

- `Insufficient`: planned pets exceed allowed pets per qualified staff for the applicable lane/time block. Group-play assignment and automatic confirmation are blocked; manager/staff schedule review is suggested.
- `Unknown`: staff schedule, roster, qualification, time-window, or policy snapshot is missing/stale. Route to staff review, not eligibility.
- `ManagerOverrideRequired`: the policy allows exceptions only with manager approval; no agent or front desk automation clears it.
- `CareModeMismatch`: a pet was counted under the wrong ratio lane, such as cat playtime under dog group play or individual day boarding under playgroup coverage.
- `IncidentHeightenedSupervision`: an incident or behavior/care note temporarily requires a stricter ratio or one-on-one handling until reviewed.
- `SplitLaneConflict`: `DayPlayPlusRoom` or late transitions create simultaneous play/enrichment and room/rest staffing obligations that cannot both be satisfied.

## 2. Domain types to add or refine

Existing types to keep and refine:

```rust
operations::daycare::StaffPetRatio
operations::daycare::StaffCount
operations::daycare::PetCount
operations::daycare::Contract
operations::daycare::EligibilityRequirement::StaffRatioAvailable
operations::daycare::GroupAssignmentRule
operations::LaborRisk
operations::OperationsRisk
operations::OperationsAction
operations::StaffTaskKind
```

Recommended new semantic surface:

```rust
operations::daycare::StaffCoveragePolicy
operations::daycare::StaffCoverageDecision
operations::daycare::StaffCoverageSnapshot
operations::daycare::StaffCoverageReason
operations::daycare::StaffCoverageRequirement
operations::daycare::StaffCoverageOverridePolicy
operations::daycare::RatioPolicyRef
operations::daycare::RatioTimeBlock
operations::daycare::QualifiedStaffCount
operations::daycare::CountedPetRoster
operations::daycare::CoverageLane
operations::daycare::CoverageLaneId
operations::daycare::CoverageCalculation
operations::daycare::StaffAssignmentSnapshot
operations::daycare::StaffQualification
operations::daycare::RosterPetCount
operations::daycare::CoverageAuditTrail
operations::daycare::CoverageRepository
```

Suggested enum and struct shapes:

```rust
pub enum CoverageLane {
    DogGroupPlay { playgroup_id: operations::daycare::PlaygroupId },
    DogIndividualDayBoarding,
    DogHybridPlayPeriod { playgroup_id: Option<operations::daycare::PlaygroupId> },
    DogHybridRoomRest,
    CatIndividualEnrichment,
}

pub enum StaffCoverageDecision {
    Sufficient {
        snapshot: StaffCoverageSnapshot,
    },
    Insufficient {
        snapshot: StaffCoverageSnapshot,
        reasons: Vec<StaffCoverageReason>,
    },
    Unknown {
        missing: Vec<StaffCoverageEvidenceGap>,
    },
    ManagerOverrideRequired {
        snapshot: StaffCoverageSnapshot,
        reasons: Vec<StaffCoverageReason>,
        allowed_by: StaffCoverageOverridePolicy,
    },
}

pub enum StaffCoverageReason {
    TooManyPetsForScheduledStaff,
    NoQualifiedStaffAssigned,
    StaffShiftDoesNotCoverTimeBlock,
    StaffBreakOrAbsenceRemovesCoverage,
    IncidentRequiresHeightenedSupervision,
    CareModeRequiresSeparateLane,
    RosterSnapshotStale,
    PolicySnapshotMissing,
}

pub struct StaffCoverageRequirement {
    pub location_id: entities::LocationId,
    pub operating_day: chrono::NaiveDate,
    pub time_block: RatioTimeBlock,
    pub lane: CoverageLane,
    pub ratio: StaffPetRatio,
    pub counted_pets: CountedPetRoster,
    pub required_qualifications: Vec<StaffQualification>,
}
```

Invariants:

- `StaffCount`, `QualifiedStaffCount`, `PetCount`, and `RosterPetCount` reject zero where the concept means a present count. If a lane has no pets, model that as `CoverageRequirement::NotNeeded` or omit the requirement rather than building a zero-pet ratio.
- `StaffPetRatio` remains non-zero on both sides and should expose both staff and pets; future calculation must not divide by raw primitives at call sites.
- `RatioTimeBlock` has a start before end and must be location-local or explicitly timezone-scoped.
- `CoverageLane` encodes care mode; it is invalid to calculate dog group-play ratio for cat enrichment or individual day boarding.
- `StaffCoverageSnapshot` records policy id/version, roster snapshot id, staff schedule snapshot id, and calculation time so audit can explain why a decision was made.
- `ManagerOverrideRequired` is not `Sufficient`; assignment/check-in remains review-gated until a manager approval event exists.

## 3. Relationship map between types

Entities and identity:

- `entities::LocationId` scopes ratio policy, staff roster, operating hours, and local overrides.
- `entities::PetId` and `entities::ReservationId` identify counted pets and tie coverage decisions to reservation/check-in decisions.
- `entities::StaffId` identifies scheduled staff, but daycare owns whether that staff member counts for a lane/time block through `StaffAssignmentSnapshot` and `StaffQualification`.

Value objects:

- `StaffPetRatio`, `StaffCount`, `PetCount`, `QualifiedStaffCount`, `RosterPetCount`, `RatioTimeBlock`, `CoverageLaneId`, and `RatioPolicyRef` carry ratio meaning without raw integers or strings.
- `CountedPetRoster` is a typed roster projection, not a generic `Vec<PetId>`; it records which reservations/pets are counted and which are excluded with semantic reasons.

Policies:

- `StaffCoveragePolicy` owns ratio evaluation.
- `GroupPlayEligibilityPolicy` consumes staff coverage decision as evidence; it must not reimplement ratio math.
- `FrontDeskThroughputPolicy` consumes coverage/readiness and produces check-in readiness; it must not override coverage.
- `IncidentPolicy` can produce heightened supervision requirements or suspend group play.

Repositories/stores:

- `PolicyRepository` loads location contract and ratio policy refs.
- `RosterRepository` loads current/planned lane rosters, playgroup assignments, rooms, waitlists, and capacity snapshots.
- `CoverageRepository` persists coverage snapshots/decisions for audit and invalidation.
- `AttendanceRepository` materializes recurring candidates whose roster impact is evaluated by coverage policy.
- Storage adapters may speak provider codes, but they must promote into these semantic values before policy evaluation.

Workflow events:

- `workflow::WorkflowEventType::BookingTriageNeeded` can trigger an initial coverage check for requested daycare.
- A future `DaycareCoverageReviewNeeded` event should carry `StaffCoverageDecision::Insufficient`, `Unknown`, or `ManagerOverrideRequired`.
- Incident escalation events can invalidate coverage snapshots when supervision requirements change.
- Reservation status update workflows may reference coverage decisions but should not mutate them silently.

Staff tasks:

- `StaffTaskKind::CheckInPrep`: created when coverage is unresolved before check-in.
- `StaffTaskKind::PlaygroupAssessment`: paired with coverage when group assignment depends on both behavior and staffed lane availability.
- `StaffTaskKind::IncidentFollowUp`: created when incidents require heightened supervision or suspension review.
- `OperationsAction::SuggestScheduleReview`: used when adding/reassigning staff could resolve coverage.
- `OperationsAction::EscalateToManager`: used for overrides or safety exceptions.

Agent specs/tools:

- `manager-daily-brief` reads coverage snapshots and drafts labor/capacity risk summaries.
- `booking-triage` can include a coverage-readiness signal in internal triage.
- `daily-care-update` must not expose staffing shortfalls to customers unless staff approve wording.
- Tools: availability lookup can read coverage status; reservation draft/update cannot confirm a covered booking unless deterministic coverage is sufficient or an approval token exists; Hermes task creation can create review tasks.

## 4. Interaction contract

Policy ownership:

```rust
impl operations::daycare::StaffCoveragePolicy {
    pub fn evaluate(
        &self,
        requirement: StaffCoverageRequirement,
        staff: StaffAssignmentSnapshot,
    ) -> operations::daycare::Result<StaffCoverageDecision>;
}
```

Behavior:

- Returns `Unknown` when any required snapshot or policy reference is missing/stale.
- Returns `Insufficient` when counted pets exceed the ratio supported by qualified overlapping staff.
- Returns `ManagerOverrideRequired` when policy allows exception consideration but requires manager approval.
- Returns `Sufficient` only when the calculation is deterministic, lane-correct, and all required qualifications/time windows are satisfied.

Roster ownership:

```rust
pub trait operations::daycare::RosterRepository {
    fn planned_roster(
        &self,
        location: entities::LocationId,
        block: RatioTimeBlock,
    ) -> operations::daycare::Result<DaycareRosterSnapshot>;

    fn lane_roster(
        &self,
        lane: CoverageLane,
        block: RatioTimeBlock,
    ) -> operations::daycare::Result<CountedPetRoster>;
}
```

Behavior:

- Builds lane-specific rosters from assignments/reservations; it does not decide whether coverage is sufficient.
- Excludes pets with typed reasons when they belong to a different care lane or are not checked in for the time block.
- Carries snapshot ids for audit/invalidation.

Policy repository ownership:

```rust
pub trait operations::daycare::PolicyRepository {
    fn daycare_contract(
        &self,
        location: entities::LocationId,
    ) -> operations::daycare::Result<operations::daycare::Contract>;

    fn staff_coverage_policy(
        &self,
        location: entities::LocationId,
        lane: CoverageLane,
    ) -> operations::daycare::Result<StaffCoveragePolicy>;
}
```

Behavior:

- Loads location and lane-specific policy. It does not inspect staff schedules or pet rosters.
- Missing local override falls back only through an explicit configured default, not an implicit hard-coded number in downstream services.

Coverage repository ownership:

```rust
pub trait operations::daycare::CoverageRepository {
    fn record_decision(
        &self,
        decision: StaffCoverageDecision,
        audit: CoverageAuditTrail,
    ) -> operations::daycare::Result<StaffCoverageSnapshotId>;

    fn current_decision(
        &self,
        location: entities::LocationId,
        lane: CoverageLane,
        block: RatioTimeBlock,
    ) -> operations::daycare::Result<Option<StaffCoverageDecision>>;

    fn invalidate_for_roster_or_staff_change(
        &self,
        change: StaffCoverageInvalidation,
    ) -> operations::daycare::Result<()>;
}
```

Behavior:

- Persists and invalidates decisions; it does not recalculate policy internally.
- Stores enough audit context to reconstruct inputs without leaking sensitive staff notes into debug output.

Assignment service interaction:

```rust
impl operations::daycare::AssignmentService {
    pub fn assign(
        &self,
        request: ReservationRequest,
        eligibility: GroupPlayEligibilityDecision,
        coverage: StaffCoverageDecision,
    ) -> operations::daycare::Result<ReservationDecision>;
}
```

Behavior:

- Dog group play requires `GroupPlayEligibilityDecision::Eligible` and `StaffCoverageDecision::Sufficient` for the chosen playgroup lane.
- Day boarding and cat individual enrichment require coverage for their individual lanes, not dog group-play coverage.
- `Unknown`, `Insufficient`, or `ManagerOverrideRequired` create review/waitlist decisions; they do not confirm assignment.

## 5. Review and approval contract

Automation level:

- Safe automation: read rosters/staff snapshots, evaluate deterministic ratio policy, record internal coverage snapshots, generate internal readiness/risk summaries, suggest staff tasks, and draft manager brief text.
- Draft-only/customer-facing: waitlist or unavailability messages may be drafted, but staff must approve wording before sending.
- Never autonomous: change staff schedules, assign staff to lanes, override ratio policy, confirm availability against insufficient/unknown coverage, or send staffing-related explanations to customers.

Review gates:

- Staff review: missing/stale roster, missing/stale staff schedule, unclear qualification, lane mismatch, late arrival/departure effects, or care note that changes supervision.
- Manager approval: any ratio override, staff schedule change, group-play assignment despite insufficient coverage, incident-heightened supervision exception, or waitlist release that depends on a policy exception.
- Customer-message approval: any owner-facing message about availability, waitlist, incident, health/safety, or schedule change.

Audit trail:

- Every persisted coverage decision records actor, trigger, policy id/version, location, time block, lane, ratio, counted pets, counted staff, excluded pets/staff with typed reasons, decision, required review gate, and approval event id if applicable.
- Agent-authored summaries must include source snapshot ids and should avoid raw staff notes or sensitive behavior details in logs/debug output.
- Overrides must preserve original deterministic result and the approving manager identity; do not overwrite `Insufficient` with `Sufficient` as if policy passed.

Customer/member-facing boundaries:

- Customers may be told only approved practical outcomes: confirmed, waitlisted, needs document/review, or staff will follow up.
- Do not expose internal staff counts, employee names, or exact ratio thresholds unless the business explicitly approves that communication policy.
- Agents must not promise availability, safety, staffing, or special supervision without deterministic coverage and required human approval.

## 6. Test contracts

Named semantic tests for the later Rust implementation:

1. `staff_pet_ratio_rejects_zero_staff_and_zero_pets`
   - `StaffCount::try_new(0)` and `PetCount::try_new(0)` fail; ratio construction remains typed.

2. `staff_coverage_policy_counts_only_qualified_staff_in_overlapping_time_block`
   - Staff outside the lane, role, qualification, break, absence, or time block do not satisfy coverage.

3. `sufficient_staff_coverage_allows_group_assignment_when_eligibility_is_eligible`
   - Eligible group-play pet and sufficient lane coverage can produce dog group-play assignment.

4. `insufficient_staff_coverage_blocks_group_assignment_even_when_temperament_is_eligible`
   - Ratio failure prevents group-play assignment and creates a manager/staff review path.

5. `unknown_staff_or_roster_snapshot_routes_to_review_not_ready`
   - Missing/stale staff schedule or roster snapshot returns `Unknown` and front desk readiness is not `Ready`.

6. `cat_individual_enrichment_uses_cat_lane_coverage_not_dog_group_play_ratio`
   - Cat playtime evaluates cat individual enrichment coverage and never requires a dog playgroup assignment.

7. `day_boarding_uses_individual_care_coverage_not_group_play_coverage`
   - A dog ineligible for group play can still proceed to individual day boarding only if individual lane coverage is sufficient.

8. `day_play_plus_room_creates_separate_play_and_room_coverage_requirements`
   - Hybrid service must satisfy both play/enrichment coverage and room/rest coverage or route to review/waitlist.

9. `incident_heightened_supervision_tightens_or_blocks_coverage_until_reviewed`
   - Incident policy can require stricter coverage or manager review; prior sufficient coverage is invalidated.

10. `manager_override_preserves_original_insufficient_decision_in_audit_trail`
    - An approved exception records the manager approval and original calculation rather than rewriting policy outcome.

11. `coverage_repository_invalidates_decision_when_roster_or_staff_schedule_changes`
    - Adding pets, removing staff, shift changes, or lane reassignment invalidates stale snapshots.

12. `front_desk_ready_requires_sufficient_coverage_and_resolved_eligibility`
    - Readiness remains non-ready if coverage is unknown/insufficient/override-required even when other requirements pass.

13. `manager_daily_brief_surfaces_ratio_risk_without_customer_facing_side_effects`
    - Brief can include labor/safety risk and suggested actions but sends no customer message and changes no schedule.

14. `storage_roundtrip_preserves_coverage_lane_ratio_policy_and_decision_variants`
    - DTO conversions preserve semantic enum variants/newtypes and reject unknown required values instead of silently defaulting.

15. `customer_waitlist_message_draft_requires_review_gate`
    - Agent-generated waitlist/unavailability text is a draft with `ReviewGate::CustomerMessageApproval`, not a sent message.

## 7. Integration notes for serialized Rust code card

Likely files touched:

- `domain/src/operations.rs`: extend `operations::daycare` with coverage policy, decision, lane, snapshot, audit, repository contracts, and errors. Consider moving daycare into `domain/src/operations/daycare.rs` if the module grows too large.
- `domain/src/policy.rs`: add/reuse review gates and approval vocabulary for ratio overrides and customer-message approval.
- `domain/src/workflow.rs`: add workflow event type(s) for daycare coverage review/invalidation if current event vocabulary is insufficient.
- `domain/src/agents.rs`: update `manager-daily-brief`, `booking-triage`, and possibly `daily-care-update` prompt packets with coverage boundaries.
- `domain/src/tools.rs`: ensure availability/reservation tools consume coverage decisions without bypassing review gates.
- `domain/tests/petsuites_core_service_contracts.rs`: extend existing daycare contract tests for coverage policy construction and invariants.
- `domain/tests/domain_quality_patterns.rs`: add semantic tests for non-zero counts, review gates, audit metadata, and no raw string/boolean ratio paths.
- Future storage/adapters: add DTO conversion tests for coverage lanes, time blocks, ratio policies, decisions, and audit snapshots.

Migration/refactor risks:

- Existing `operations::daycare::StaffPetRatio` is a simple value object; adding policy decisions should not turn it into an anemic helper with scattered free-function math. Put evaluation on `StaffCoveragePolicy`.
- Top-level `operations::ScheduledStaffCount` and `LaborSnapshot` are manager-brief concepts, not daycare lane coverage by themselves. Avoid reusing them as if they prove daycare supervision.
- `GroupAssignmentRule` currently mixes rule and assignment concepts. Coverage work should split policy/rule from concrete `PlaygroupAssignment` and lane coverage snapshots.
- `EligibilityRequirement::StaffRatioAvailable` is a requirement marker; it should become evidence consumed by eligibility/assignment services, not the place where coverage is calculated.
- Care mode distinctions are easy to flatten. Keep cat enrichment, individual day boarding, dog group play, and hybrid room/play coverage separate.
- Debug/log output must avoid leaking staff notes, sensitive behavior details, or raw customer/member-facing content.

Dependencies on other implications:

- Group-play eligibility: coverage is required evidence for group-play assignment but does not decide temperament/vaccine/spay-neuter suitability.
- Playgroup assignment: assignment consumes coverage and eligibility; it should not own ratio math.
- Daily recurring attendance: recurring visits create future roster load and must materialize explicit coverage checks rather than silently booking.
- Incident handling: incidents can invalidate coverage or require heightened supervision until manager review.
- Front desk readiness: readiness consumes coverage status and review gates before check-in.
- Package/membership opportunity: package recommendations may use attendance patterns but must not ignore future ratio/capacity risk.

Implementation posture:

- Start with contract tests for `StaffCoveragePolicy::evaluate` and storage roundtrip behavior.
- Add semantic types before wiring tools or agents.
- Keep live operational changes behind review gates. The first Rust card should be contract/modeling only unless separately authorized.
