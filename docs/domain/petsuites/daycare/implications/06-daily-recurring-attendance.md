# Daycare implication 06: Daily recurring attendance

Purpose: model daily recurring daycare attendance as a first-class operational workflow, not as a calendar convenience flag. The recurring plan is a promise to prepare explicit dated attendance candidates; it is not permission to silently book, check in, consume prepaid visits, charge a customer, or bypass eligibility/capacity review.

Assumptions:

- PetSuites daycare recurrence usually means a pet attends on a predictable weekly pattern, often tied to a prepaid package, membership, or customer habit.
- Source systems may represent recurrence as free-text notes, repeating reservations, package balances, membership labels, or cloned calendar entries. Domain code should promote those facts into typed recurrence and occurrence values before policy decisions.
- Missing schedule, eligibility, package, payment, or capacity facts create review/waitlist states rather than confirmed attendance.
- This card is modeling-only; no live reservations, customer messages, package changes, or payment actions are performed.

## 1. Operational story

### Trigger

A daily recurring attendance workflow begins when one of these events is observed:

- A customer asks for repeating daycare attendance, such as every weekday, Mondays/Wednesdays/Fridays, or a fixed day for a known term.
- Staff converts a repeated daycare habit into a managed schedule.
- A package or membership purchase implies expected future attendance but does not yet define all occurrence dates.
- A manager/front desk review identifies repeated manual bookings that should be represented as a recurrence plan.
- An existing recurrence reaches a renewal boundary, exception, missed-visit threshold, package depletion warning, eligibility invalidation, or capacity conflict.

### Actors

- Customer or member: requests recurring attendance, approves schedule/payment terms, receives staff-approved updates.
- Front desk: captures request, verifies package/membership/payment state, handles exceptions, and confirms occurrence readiness.
- Daycare lead/playgroup staff: reviews group-play eligibility, playgroup assignment, attendance capacity, and care notes.
- Manager: approves exceptions, overrides, suspension/reinstatement, package corrections, capacity conflicts, or sensitive customer-facing communication.
- Booking triage agent: extracts recurrence intent and drafts missing-fact tasks.
- Manager daily brief agent: summarizes recurring attendance risks and package/membership opportunities.
- Daily care update agent: drafts approved occurrence updates after staff notes/photos exist.

### Inputs

- `entities::CustomerId`, `entities::PetId`, `entities::LocationId`.
- Requested `operations::daycare::ServiceVariant` and derived `operations::daycare::CareMode`.
- `operations::daycare::AttendanceRecurrence`: typed days/rule, start date, end condition, holiday/blackout behavior, customer-requested exceptions, and source.
- `operations::daycare::AttendanceWindow`: arrival/departure expectation for each occurrence.
- Location contract from `operations::daycare::PolicyRepository`: attendance policy, local hours, package policy, staff ratio, eligibility requirements, blackout/closure rules, and recurrence limits.
- Current eligibility evidence: temperament, vaccines, spay/neuter facts where applicable, incident restrictions, age thresholds, care/medical review requirements.
- Capacity and staffing snapshots for each occurrence date: playgroup/room lane capacity, staff coverage, existing reservations, waitlist state.
- Package/membership/payment context: package balance, term dates, membership status, package expiration, payment holds, and checkout requirements.
- Prior attendance history: missed visits, no-shows, cancellations, completed check-ins, consumed package visits, and recurring-pattern reliability.

### Decisions

The workflow makes typed decisions for the plan and for each occurrence:

1. Is the recurrence definition complete enough to materialize dated occurrence candidates?
2. Is the requested service compatible with the pet species, care mode, location policy, and recurrence rule?
3. Does group play require current eligibility review before occurrences can be confirmed?
4. Does each occurrence have deterministic capacity and staff coverage, or should it become waitlisted/review-needed?
5. Does the package/membership/payment state support the occurrence, require front-desk review, or only create a revenue opportunity?
6. Are there exceptions, holidays, closures, blackout dates, incident restrictions, care-note changes, or customer-requested skips that modify occurrence state?
7. Has attendance history crossed thresholds for package suggestion, renewal reminder, missed-visit follow-up, or manager review?

### Outputs

- `operations::daycare::DailyRecurringAttendance`: the recurrence plan aggregate with owner identities, service variant, care mode, recurrence rule, term, package/membership link, status, audit source, and review gates.
- `operations::daycare::AttendanceOccurrenceCandidate` values: one dated candidate per materialized day, each with eligibility, capacity, staff coverage, package/payment readiness, and review state.
- `operations::daycare::RecurringAttendanceDecision`: accepted-for-draft, needs staff review, needs manager review, waitlisted, rejected/incompatible, suspended, expired, or canceled.
- `operations::StaffTask` recommendations for playgroup assessment, document review, check-in prep, customer follow-up, package review, incident follow-up, or manager capacity review.
- `operations::RevenueOpportunity { opportunity: DaycarePackageCandidate }` when attendance pattern suggests a package or membership; recommendation only.
- `workflow::WorkflowEvent` records for recurrence drafted, occurrence materialized, exception applied, eligibility invalidated, package warning, review requested, confirmation approved, or recurrence canceled.
- Customer message drafts only when review gates require and preserve customer-safe wording.

### Success state

A successful recurrence is explicit, reviewable, and safe:

- The recurrence plan has a typed rule, start/end or renewal boundary, service variant, care mode, location, customer, pet, and source.
- Materialized occurrences are explicit candidates or reservations with per-date readiness decisions; no invisible calendar mutation occurs.
- Group-play occurrences are not confirmed until current eligibility, capacity, staff ratio, and playgroup assignment are satisfied.
- Individual/day boarding/cat enrichment occurrences use their own capacity/care-lane rules, not dog group-play assumptions.
- Package/membership/payment implications are represented as typed readiness/recommendation states, not autonomous charges or enrollment.
- Audit history shows who/what created, reviewed, approved, modified, skipped, confirmed, or canceled each plan/occurrence.

### Failure and exception states

- `IncompleteRecurrence`: missing days, date bounds, location, customer/pet identity, service variant, or attendance window.
- `UnsupportedRecurrenceRule`: source recurrence is too vague or too complex to convert safely without staff review.
- `EligibilityReviewRequired`: missing/stale temperament, vaccine, spay/neuter, care-note, age, or incident-clearance evidence.
- `GroupPlaySuspended`: incident or manager decision blocks group play; individual care may still be separately considered.
- `CapacityWaitlist`: occurrence date lacks room/playgroup capacity or staff coverage.
- `PackageReviewRequired`: package balance, membership term, expiration, or payment state is ambiguous or insufficient.
- `HolidayOrClosureException`: date falls outside operating rules and must be skipped, waitlisted, or manager-approved.
- `CustomerScheduleException`: customer-requested skip/change is captured without silently consuming a visit.
- `MissedVisitReview`: no-show/missed attendance crosses a location-defined review threshold.
- `SourceSystemConflict`: imported repeated reservations disagree with the domain recurrence plan.
- `CustomerMessageBlocked`: a customer-facing update exists only as a draft until approval.

## 2. Domain types to add or refine

### Aggregate and plan identity

```rust
operations::daycare::recurring_attendance::Id
operations::daycare::DailyRecurringAttendance
operations::daycare::RecurringAttendanceStatus
operations::daycare::RecurringAttendanceSource
operations::daycare::RecurringAttendanceTerm
operations::daycare::RecurringAttendanceAuditEntry
```

Invariants:

- `recurring_attendance::Id` is a non-empty source-neutral ID; provider IDs convert at adapter boundaries.
- `DailyRecurringAttendance` requires customer, pet, location, service variant, care mode, recurrence, attendance window, term, source, status, and audit source.
- `RecurringAttendanceTerm` has a start date and either an end date, package/membership term reference, or explicit manager-reviewed open-ended renewal rule.
- A recurrence cannot be `Active` if its latest deterministic decision is `NeedsStaffReview`, `NeedsManagerReview`, `Suspended`, `Expired`, or `Canceled`.
- Status is an enum, never `active: bool` plus loose reason strings.

Suggested status shape:

```rust
operations::daycare::RecurringAttendanceStatus::{
    Draft,
    PendingCustomerApproval,
    PendingStaffReview,
    PendingManagerReview,
    Active,
    Waitlisted,
    Suspended { reason: RecurringAttendanceSuspensionReason },
    Expired,
    Canceled { reason: RecurringAttendanceCancellationReason },
}
```

### Recurrence rule and dates

```rust
operations::daycare::AttendanceRecurrence
operations::daycare::AttendanceDays
operations::daycare::AttendanceFrequency
operations::daycare::AttendanceWindow
operations::daycare::AttendanceOccurrenceDate
operations::daycare::AttendanceException
operations::daycare::HolidayBlackoutBehavior
operations::daycare::MaterializationHorizon
```

Invariants:

- `AttendanceDays` is non-empty and uses typed weekdays/dates, not free text.
- `AttendanceRecurrence` requires an explicit start date and either an end date, occurrence count, package/membership term, or reviewed renewal boundary.
- `AttendanceWindow` has a start and end time in the location timezone; end must be after start for same-day daycare.
- `MaterializationHorizon` limits how far future candidates are created; it prevents unbounded recurring reservation creation.
- Exceptions are explicit values: skipped date, one-time date change, holiday closure, capacity blackout, customer request, manager override, or source-system conflict.

### Occurrence candidates and attendance state

```rust
operations::daycare::AttendanceOccurrenceCandidate
operations::daycare::AttendanceOccurrenceDecision
operations::daycare::AttendanceOccurrenceReadiness
operations::daycare::AttendanceOccurrenceStatus
operations::daycare::CheckInReadiness
operations::daycare::MissedVisitDisposition
```

Invariants:

- Occurrence candidates are dated, traceable to a recurrence plan, and never imply checked-in status by existence alone.
- `AttendanceOccurrenceDecision::Confirmable` requires eligibility, capacity, staff coverage, package/payment readiness, and care-note readiness.
- `CheckedIn` is only reachable from a concrete occurrence/reservation after front-desk action or approved integration event.
- A missed/skipped occurrence does not consume a package visit unless payment/package policy explicitly says so and review gates are satisfied.

### Package, membership, and consumption values

```rust
operations::daycare::PackageBalance
operations::daycare::PackageVisitLedgerEntry
operations::daycare::PackageConsumptionDecision
operations::daycare::MembershipAttendanceEntitlement
operations::daycare::RecurringAttendanceRevenueSignal
```

Invariants:

- Package balance cannot be negative; consumption/refund/correction entries are typed ledger events.
- Package consumption is a decision tied to a confirmed or completed occurrence, not to recurrence materialization.
- Membership entitlement has a term and service/visit constraints; it cannot be treated as unlimited availability unless policy explicitly says so.
- Revenue signals are recommendations; they never enroll, discount, refund, charge, or promise pricing.

### Policy, error, and audit types

```rust
operations::daycare::RecurringAttendancePolicy
operations::daycare::RecurringAttendanceDecision
operations::daycare::RecurringAttendanceReviewReason
operations::daycare::RecurringAttendanceDenialReason
operations::daycare::recurring_attendance::Error
operations::daycare::recurring_attendance::Result<T>
```

Invariants:

- Unknown or stale required facts produce review reasons, not eligibility or confirmation.
- Denial reasons are typed: unsupported species/service, recurrence outside operating dates, hard eligibility stop, closed location, capacity unavailable beyond waitlist policy, or package/payment hard stop.
- Errors preserve the domain path; boundary layers may translate them into API/storage errors later.

## 3. Relationship map between types

### Entities

- `entities::CustomerId`: owns the human account requesting/approving the schedule; does not own recurrence policy.
- `entities::PetId`: recurrence is per pet because eligibility, care notes, incidents, and attendance history are pet-specific.
- `entities::LocationId`: scopes hours, blackout dates, staff ratios, package/membership availability, and approval policy.
- `entities::ReservationId`: may be created for a confirmed occurrence, but the recurrence plan exists before and above individual reservations.
- `operations::daycare::recurring_attendance::Id`: stable recurrence-plan identity.

### Value objects

- `AttendanceRecurrence`, `AttendanceDays`, `AttendanceWindow`, `RecurringAttendanceTerm`, `AttendanceOccurrenceDate`, `MaterializationHorizon`.
- `PackageBalance`, `MembershipAttendanceEntitlement`, `PackageVisitLedgerEntry`.
- `AssignmentRationale`, `EligibilityNote`, and recurrence review notes should be redacted in debug output when they can contain staff/customer-sensitive details.

### Policies

- `RecurringAttendancePolicy`: validates recurrence shape, term bounds, allowed service/care mode, materialization horizon, missed-visit handling, and exception behavior.
- `GroupPlayEligibilityPolicy`: evaluates whether the pet may join dog group play for the relevant occurrence period.
- `StaffCoveragePolicy`: evaluates per-date ratio/capacity/staff coverage.
- `PackagePolicy`: decides whether an occurrence is pay-per-visit, package-backed, membership-backed, or needs review.
- `FrontDeskThroughputPolicy`: converts occurrence readiness into check-in prep states.

### Repositories and stores

- `operations::daycare::RecurringAttendanceRepository`: stores recurrence plans, audit entries, status transitions, and source-system correlation IDs.
- `operations::daycare::AttendanceRepository`: reads/writes occurrence candidates, reservations, check-in/out facts, missed visits, and exception applications.
- `operations::daycare::PolicyRepository`: loads location service contract and recurrence/package/holiday policies.
- `operations::daycare::RosterRepository`: provides dated capacity/staff/playgroup/room snapshots.
- `operations::daycare::EligibilityRepository`: reads/writes eligibility evidence snapshots and invalidation events.
- `operations::daycare::PackageLedgerRepository`: reads package/membership entitlements and appends typed consumption/correction recommendations.

Storage DTOs may contain raw provider codes and cloned reservation records. They must convert into semantic recurrence, occurrence, package, and decision values before domain policy evaluates them.

### Workflow events

```rust
workflow::WorkflowEventType::{
    DaycareRecurringAttendanceDrafted,
    DaycareRecurringAttendanceReviewRequested,
    DaycareOccurrenceCandidatesMaterialized,
    DaycareOccurrenceWaitlisted,
    DaycareOccurrenceConfirmed,
    DaycareAttendanceExceptionApplied,
    DaycareRecurringAttendanceEligibilityInvalidated,
    DaycarePackageBalanceReviewRequested,
    DaycareRecurringAttendanceCanceled,
}
```

If the current `WorkflowEventType` enum remains generic, these can initially be represented by typed payloads behind existing booking-triage/status-update events. The later code card should prefer explicit daycare event variants once the workflow module is extended.

### Staff tasks

- `StaffTaskKind::CheckInPrep { reservation_id }`: occurrence is materially ready but needs day-of front-desk execution.
- `StaffTaskKind::PlaygroupAssessment { pet_id }`: group-play eligibility/assignment cannot be resolved by automation.
- `StaffTaskKind::DocumentReview { pet_id }`: vaccine/spay-neuter/package proof needs review.
- `StaffTaskKind::IncidentFollowUp { pet_id }`: incident invalidated recurrence eligibility or occurrence readiness.
- `StaffTaskKind::CustomerFollowUp { customer_id, reason }`: customer approval, missing facts, package warning, or schedule exception requires staff-controlled outreach.
- Future refinement: `StaffTaskKind::RecurringAttendanceReview { recurrence_id }` if generic tasks become too lossy.

### Agent specs and tools

- `agents::booking-triage`: extract recurrence intent, service variant, dates, and missing facts; draft only.
- `agents::manager-daily-brief`: summarize recurring attendance exceptions, waitlist pressure, package depletion, and missed-visit patterns.
- `agents::daily-care-update`: draft occurrence updates after staff-approved notes/photos exist.
- `agents::lead-conversion`: classify package/membership opportunity from repeated attendance patterns; recommendation only.
- `tools::availability_lookup`: read capacity and hours.
- `tools::reservation_draft` / reservation update tools: draft occurrence candidates/reservations only within approval gates.
- `tools::payment` and package tooling: read or draft review packets; no autonomous capture, refund, discount, or enrollment.
- `tools::messaging`: draft only until customer-message approval.
- `tools::hermes_task`: create internal review tasks for staff/manager queues.

## 4. Interaction contract

Use truthful owners: recurrence behavior belongs on recurrence plans, recurrence policies, attendance materializers, and daycare-owned repositories/services. Avoid free-floating helpers such as `utils::make_recurring_reservations`.

### Recurrence construction

```rust
impl operations::daycare::DailyRecurringAttendance {
    pub fn builder() -> DailyRecurringAttendanceBuilder<MissingRequiredFields>;

    pub fn request_activation(
        self,
        policy: &operations::daycare::RecurringAttendancePolicy,
        context: operations::daycare::RecurringAttendanceActivationContext,
    ) -> operations::daycare::recurring_attendance::Result<
        operations::daycare::RecurringAttendanceDecision,
    >;

    pub fn apply_exception(
        &mut self,
        exception: operations::daycare::AttendanceException,
        actor: entities::ActorRef,
    ) -> operations::daycare::recurring_attendance::Result<
        operations::daycare::RecurringAttendanceAuditEntry,
    >;

    pub fn suspend_for(
        &mut self,
        reason: operations::daycare::RecurringAttendanceSuspensionReason,
        actor: entities::ActorRef,
    ) -> operations::daycare::RecurringAttendanceAuditEntry;
}
```

Required builder fields:

- customer, pet, location;
- service variant and derived care mode;
- recurrence rule, term, attendance window;
- source and created-by actor;
- optional package/membership reference, imported source ID, and customer notes.

### Policy validation

```rust
impl operations::daycare::RecurringAttendancePolicy {
    pub fn evaluate_plan(
        &self,
        plan: &operations::daycare::DailyRecurringAttendance,
        location_contract: &operations::daycare::Contract,
        customer_context: &operations::daycare::CustomerAttendanceContext,
        pet_context: &operations::daycare::PetAttendanceContext,
    ) -> operations::daycare::RecurringAttendanceDecision;

    pub fn evaluate_occurrence(
        &self,
        candidate: &operations::daycare::AttendanceOccurrenceCandidate,
        readiness: operations::daycare::OccurrenceReadinessContext,
    ) -> operations::daycare::AttendanceOccurrenceDecision;
}
```

Behavior:

- Return `NeedsStaffReview` for unknown/ambiguous recurrence facts.
- Return `NeedsManagerReview` for policy overrides, open-ended recurrence without renewal bounds, hard eligibility exceptions, package corrections, or capacity exceptions.
- Return `Denied` only for typed hard stops; include denial reasons.
- Never create reservations, consume package visits, charge payment, or send messages.

### Materialization service

```rust
pub trait operations::daycare::AttendanceMaterializer {
    fn materialize(
        &self,
        plan: &operations::daycare::DailyRecurringAttendance,
        horizon: operations::daycare::MaterializationHorizon,
        calendar: &dyn operations::daycare::LocationCalendarRepository,
        policies: &dyn operations::daycare::PolicyRepository,
    ) -> operations::daycare::recurring_attendance::Result<
        Vec<operations::daycare::AttendanceOccurrenceCandidate>,
    >;
}
```

Behavior:

- Emits explicit occurrence candidates within the horizon.
- Applies holidays/blackouts/exceptions as typed candidate states.
- Deduplicates against existing occurrence/reservation correlation IDs.
- Does not confirm, check in, or bill.

### Occurrence readiness service

```rust
pub struct operations::daycare::OccurrenceReadinessService<Rosters, Eligibility, Packages> {
    rosters: Rosters,
    eligibility: Eligibility,
    packages: Packages,
}

impl<Rosters, Eligibility, Packages> OccurrenceReadinessService<Rosters, Eligibility, Packages> {
    pub fn evaluate(
        &self,
        candidate: &operations::daycare::AttendanceOccurrenceCandidate,
    ) -> operations::daycare::recurring_attendance::Result<
        operations::daycare::AttendanceOccurrenceReadiness,
    >;
}
```

Behavior:

- Delegates group-play truth to `GroupPlayEligibilityPolicy` and `AssignmentService`.
- Delegates staff/capacity truth to `StaffCoveragePolicy` and `RosterRepository`.
- Delegates package/payment truth to package ledger/payment readiness contracts.
- Aggregates readiness into confirmable/review/waitlist states without hiding underlying reasons.

### Repository contracts

```rust
pub trait operations::daycare::RecurringAttendanceRepository {
    fn get(
        &self,
        id: operations::daycare::recurring_attendance::Id,
    ) -> operations::daycare::recurring_attendance::Result<
        operations::daycare::DailyRecurringAttendance,
    >;

    fn save_draft(
        &self,
        plan: &operations::daycare::DailyRecurringAttendance,
    ) -> operations::daycare::recurring_attendance::Result<()>;

    fn append_audit(
        &self,
        id: operations::daycare::recurring_attendance::Id,
        entry: operations::daycare::RecurringAttendanceAuditEntry,
    ) -> operations::daycare::recurring_attendance::Result<()>;

    fn plans_requiring_review(
        &self,
        location: entities::LocationId,
        day: operations::ResortOperatingDay,
    ) -> operations::daycare::recurring_attendance::Result<
        Vec<operations::daycare::DailyRecurringAttendance>,
    >;
}

pub trait operations::daycare::AttendanceRepository {
    fn existing_occurrences_for_plan(
        &self,
        id: operations::daycare::recurring_attendance::Id,
        horizon: operations::daycare::MaterializationHorizon,
    ) -> operations::daycare::recurring_attendance::Result<
        Vec<operations::daycare::AttendanceOccurrenceCandidate>,
    >;

    fn upsert_occurrence_candidates(
        &self,
        candidates: Vec<operations::daycare::AttendanceOccurrenceCandidate>,
    ) -> operations::daycare::recurring_attendance::Result<()>;

    fn mark_missed_visit(
        &self,
        occurrence: operations::daycare::AttendanceOccurrenceId,
        disposition: operations::daycare::MissedVisitDisposition,
    ) -> operations::daycare::recurring_attendance::Result<()>;
}
```

Repository methods return semantic domain values and module-local errors. Adapter implementations may translate source-system records, but raw provider recurrence strings should not leak into callers.

### Agent/tool interaction contract

```rust
operations::daycare::RecurringAttendanceAgentPacket {
    workflow_event: workflow::WorkflowEvent,
    plan: Option<operations::daycare::DailyRecurringAttendance>,
    candidates: Vec<operations::daycare::AttendanceOccurrenceCandidate>,
    review_reasons: Vec<operations::daycare::RecurringAttendanceReviewReason>,
    allowed_actions: Vec<workflow::AllowedAction>,
    required_reviews: Vec<policy::ReviewGate>,
}
```

Agents may:

- extract recurrence intent;
- draft a plan and occurrence candidates;
- summarize review reasons;
- create internal tasks;
- draft customer follow-up text.

Agents must not:

- confirm a reservation without deterministic availability and required review;
- check in a pet;
- consume a package visit;
- capture/refund/discount/payment;
- enroll membership;
- override group-play eligibility, staff ratio, incident restrictions, or local policy;
- send customer-facing messages without approval.

## 5. Review and approval contract

### Automation level

Default automation level: `policy::AutomationLevel::DraftOnly`.

Allowed autonomous actions:

- Parse recurrence intent from staff/customer/source-system text into a draft `AttendanceRecurrence`.
- Materialize occurrence candidates within a configured horizon.
- Read policy, capacity, eligibility, package, and attendance history.
- Produce readiness summaries and internal review task recommendations.
- Draft customer-safe follow-up copy with unresolved requirements clearly stated.

### Staff review gates

Require staff review for:

- activating a new recurring attendance plan;
- clearing missing or stale eligibility evidence;
- customer-requested schedule exceptions that change occurrence dates/windows;
- package/membership ambiguity or depletion warnings that require customer discussion;
- day-of check-in readiness with unresolved care notes, vaccine proof, or assignment details;
- converting draft occurrence candidates into actual reservations when policy requires manual confirmation.

### Manager approval gates

Require manager/human approval for:

- overriding capacity, staff ratio, holiday/blackout, local recurrence-limit, or waitlist policy;
- suspending or reinstating group-play eligibility after incidents;
- open-ended recurrence without explicit renewal boundary;
- package ledger corrections, refunds, waivers, discounts, membership enrollment, or payment capture;
- customer-facing incident/health/safety messages;
- any staff schedule change recommended by recurring attendance pressure.

### Audit trail

Every recurrence plan and occurrence decision should retain:

- actor (`entities::ActorRef`) and automation level;
- source event/tool/import ID;
- before/after status;
- typed decision/reason values;
- review gate satisfied or still required;
- timestamp and location;
- customer-message draft ID or staff task ID when created;
- package ledger recommendation ID when package state is affected.

The audit trail must distinguish draft/recommendation from approved operational changes. It must never store raw secrets, payment credentials, or unredacted sensitive care/behavior notes in generic debug fields.

### Customer/member-facing boundaries

- A recurrence draft is not a confirmed schedule until staff/customer approvals are satisfied.
- A package recommendation is not a package sale or price promise.
- Occurrence candidates are not check-ins.
- Missed/skipped visits are not automatically charged or consumed unless explicit package/payment policy and review gates permit it.
- Incident, health, and behavior details are routed through configured approval gates before customer delivery.

## 6. Test contracts

Named semantic tests for later implementation:

1. `recurring_attendance_requires_non_empty_typed_attendance_days`
   - Building an `AttendanceRecurrence` with no days or only blank source text fails with a recurrence-domain error.

2. `recurring_attendance_requires_start_and_end_or_reviewed_renewal_boundary`
   - A recurrence cannot become active with no start or no term/renewal bound unless manager review explicitly approves an open-ended plan.

3. `materialization_creates_explicit_occurrence_candidates_not_confirmed_checkins`
   - Materializing a weekly recurrence emits dated occurrence candidates whose status is draft/review/waitlist/confirmable, never checked-in.

4. `materialization_horizon_prevents_unbounded_future_reservations`
   - The materializer refuses or truncates requests beyond the configured horizon and records the decision.

5. `recurrence_exceptions_skip_or_move_only_named_occurrences`
   - A customer holiday skip or one-time date move affects only the targeted occurrence and is visible in the audit trail.

6. `holiday_blackout_creates_exception_or_review_not_silent_booking`
   - Occurrences falling on closures/blackouts become skipped/waitlisted/review-required according to policy.

7. `group_play_recurring_occurrence_requires_current_eligibility_each_materialization_window`
   - Group-play candidates with stale temperament/vaccine/spay-neuter/incident evidence are not confirmable.

8. `day_boarding_recurrence_can_continue_when_group_play_is_suspended_if_individual_care_is_safe`
   - Group-play suspension does not automatically cancel individual day boarding when capacity/care policy permits it.

9. `capacity_waitlist_occurrence_preserves_plan_without_promising_availability`
   - A capacity conflict marks the occurrence waitlisted while preserving the recurrence plan and creating review/manager signals.

10. `staff_ratio_shortfall_blocks_confirmable_group_play_occurrence`
    - Insufficient staff coverage prevents group-play occurrence confirmation and creates/suggests staff or manager review.

11. `package_visit_is_consumed_only_after_confirmed_or_completed_occurrence_policy`
    - Draft/materialized/skipped/waitlisted occurrences do not decrement `PackageBalance`.

12. `missed_visit_disposition_does_not_charge_without_policy_and_review_gate`
    - Missed visits produce typed disposition and follow-up; charge/consumption requires explicit policy and approval.

13. `membership_entitlement_has_term_and_service_constraints`
    - Membership-backed recurrence cannot authorize dates outside the term or for unsupported service variants.

14. `attendance_history_package_candidate_is_recommendation_not_payment_action`
    - Repeated pay-per-visit attendance can emit `DaycarePackageCandidate`; no payment/enrollment action is created.

15. `source_system_cloned_reservations_deduplicate_against_recurrence_plan`
    - Imported cloned reservations correlate to occurrence candidates instead of creating duplicates.

16. `recurrence_status_is_semantic_enum_not_boolean_active_flag`
    - Status transitions use typed variants and reject impossible transitions such as `Active` with unresolved manager review.

17. `recurring_attendance_audit_records_actor_source_review_gate_and_decision_reason`
    - Every activation, exception, suspension, package warning, and cancellation produces an audit entry with typed context.

18. `customer_message_draft_for_recurring_attendance_requires_customer_message_approval`
    - Follow-up drafts include required review gates and are not sent by the agent/tool contract.

19. `storage_codecs_reject_invalid_recurrence_and_package_values`
    - Storage adapters reject empty recurrence days, zero/negative package balances, unknown hard-stop enum values, and ambiguous dates rather than defaulting silently.

20. `public_paths_preserve_daycare_recurring_attendance_meaning`
    - API examples use `operations::daycare::DailyRecurringAttendance`, `operations::daycare::AttendanceRecurrence`, and `operations::daycare::recurring_attendance::Id` rather than generic `Schedule`, `RecurringBooking`, or raw strings.

## 7. Integration notes for serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add or split `operations::daycare::recurring_attendance` types.
  - Refine existing `AttendancePolicy`, `PackagePolicy`, and `Contract` to reference recurrence/materialization/package readiness contracts.
  - Add recurrence/occurrence status, decision, review reason, denial reason, and error types.
- `domain/src/reservation/mod.rs`
  - Add conversion/adaptation points between daycare occurrence candidates and reservation drafts.
  - Preserve same-day daycare windows separately from overnight boarding semantics.
- `domain/src/policy.rs`
  - Add review gates or automation-policy values if recurring attendance needs more precise staff/customer/manager approval gates.
- `domain/src/workflow.rs`
  - Add explicit daycare recurring attendance workflow event variants or typed payload contracts.
- `domain/src/agents.rs`
  - Extend booking-triage, manager-daily-brief, daily-care-update, and lead-conversion packets for recurrence/readiness/package signals.
- `domain/src/tools.rs`
  - Ensure tool contracts distinguish draft occurrence/reservation, confirmation, payment/package operations, and customer messaging boundaries.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add contract-level recurrence examples and non-zero/semantic enum assertions.
- `domain/tests/domain_quality_patterns.rs`
  - Add semantic path, audit, debug-redaction, and review-gate tests for recurrence values.
- Future storage crate/adapters
  - Add provider-code conversion and deduplication tests for imported recurring reservations, package ledgers, and occurrence correlation IDs.

### Migration and refactor risks

- Current `operations::daycare::AttendancePolicy` is high-level. Do not overload it with recurrence state; add `AttendanceRecurrence` and occurrence decision types.
- Current `PackagePolicy::Membership` has no term/entitlement semantics. Later code must avoid treating it as unlimited or confirmed payment.
- `PackageVisits` is positive, but package balances and ledger entries need separate types because balances can reach zero while package definitions cannot.
- Reusing `entities::Reservation` too early may collapse recurrence plans into cloned reservations. Keep the recurrence aggregate separate and emit explicit occurrence candidates/reservation drafts.
- Group-play eligibility decisions may vary over time. Materialization should re-evaluate or snapshot eligibility per horizon rather than assuming the plan's original eligibility remains true forever.
- Imported source-system recurrence/free-text notes may be ambiguous. Adapter conversion should produce review reasons, not guessed recurrence rules.
- Debug output for notes/rationales must remain redacted; recurrence audit should store typed reasons and IDs rather than unbounded staff/customer notes.
- Avoid generic schedule helpers or flattened types. The public path should communicate `operations::daycare::recurring_attendance` and `operations::daycare::AttendanceOccurrence`.

### Dependencies on other implications

- Group-play eligibility and playgroup assignment implications: recurring group-play occurrences must consume eligibility and assignment decisions, not duplicate those policies.
- Package/membership opportunity implication: package suggestions and ledgers must stay recommendation/review-gated until approved.
- Front-desk readiness/check-in implication: occurrence readiness feeds check-in prep, but check-in remains a separate day-of operational action.
- Incident handling implication: incidents can invalidate recurrence plans, suspend group play, and require customer/manager review.
- Daily care update implication: recurring attendance produces frequent update opportunities, but customer updates require staff-approved occurrence notes/photos.

### Implementation order recommendation

1. Add doc-backed semantic tests for recurrence rule, occurrence materialization, status transitions, and package non-consumption.
2. Introduce `operations::daycare::recurring_attendance` IDs, errors, status, recurrence, term, exception, and occurrence candidate types.
3. Add `RecurringAttendancePolicy` and `AttendanceMaterializer` contracts with conservative review-first behavior.
4. Add repository traits for recurrence, occurrence, policy, roster, eligibility, and package ledger reads/writes.
5. Wire agent/workflow packets as draft/recommendation-only surfaces.
6. Add storage/adapter conversion tests for provider recurrence strings, cloned reservations, package balances, and occurrence correlation IDs.

Doc-only status: this artifact defines the intended domain model and operational contract for daily recurring attendance. It does not modify code, schemas, live systems, reservations, packages, payments, or customer-facing communication.
