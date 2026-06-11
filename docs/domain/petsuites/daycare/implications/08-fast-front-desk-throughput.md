# Daycare operational implication 08: Fast front-desk throughput

Purpose: make daycare check-in/check-out fast without allowing speed to erase safety, payment, care-note, or customer-facing approval boundaries. This implication turns `operations::daycare::FrontDeskThroughputPolicy` from the service-domain map into a semantic readiness workflow that front-desk staff, kennel/play staff, managers, and approved agents can share.

Modeling assumption: PetSuites-style daycare throughput is safest when the front desk sees a small set of typed readiness outcomes before the customer arrives. Unknown or stale facts should create pre-arrival staff work or a review lane, not a silent green check. This document is modeling-only; it does not change code, storage schemas, live systems, reservations, customer messages, payments, or policy.

## 1. Operational story

### Trigger

`operations::daycare::front_desk::ThroughputWorkflow` starts when one of these events occurs:

- A daycare reservation is created, changed, or materialized from recurring attendance.
- The operating-day prep job opens the next-day or same-day check-in queue.
- A customer arrives for daycare check-in or asks the front desk to add/change the service.
- A checkout/payment/package-consumption queue is opened for pets leaving daycare.
- A staff note, incident, vaccine update, package balance change, or capacity/staffing change invalidates an earlier readiness decision.

### Actors

- Customer/member: provides pet, reservation, package/payment, vaccine, and care-instruction facts; receives only approved customer-facing outputs.
- Front-desk staff: owns identity matching, arrival/checkout interaction, requirement collection, package/payment presentation, and final customer-facing confirmation.
- Kennel/play staff or lead: owns physical handoff, group-play assessment, room/yard assignment, care-note review, and unresolved handling questions.
- Manager: owns overrides for hard stops, capacity/ratio exceptions, incident restrictions, refunds/waivers/discounts, and sensitive owner communications.
- AI agent/runtime: may read facts, assemble readiness packets, draft internal tasks/messages, and recommend queue order; it must not confirm unsafe check-ins, override policy, charge/refund/enroll, or send sensitive customer messages autonomously.

### Inputs

The workflow consumes a typed `operations::daycare::front_desk::ReadinessContext` rather than a pile of raw booleans:

- Reservation identity: `reservation::Id`, `entities::CustomerId`, `entities::PetId`, `entities::LocationId`, requested date/window, source-system correlation ID where present.
- Requested daycare service: `operations::daycare::ServiceVariant` and derived `operations::daycare::CareMode`.
- Operating-day queue facts: expected arrival/departure windows, current check-in/check-out queue position, and physical handoff lane.
- Eligibility facts: current `GroupPlayEligibilityDecision`, vaccine proof status, spay/neuter status where relevant, age threshold, temperament evidence, and unresolved incident restrictions.
- Assignment/capacity facts: `PlaygroupAssignment` or room/enrichment lane, `StaffCoverageDecision`, current roster capacity, and waitlist state.
- Care facts: feeding/medication/allergy/medical/behavior note requirements, emergency-contact gaps, and staff-review requirements from `care`.
- Money/package facts: package/membership entitlement, visit balance, payment due timing, checkout charges, refund/credit/waiver flags, and payment authorization state.
- Customer communication facts: approved follow-up scripts, unapproved drafts, opt-in/contact preferences, and review gate state.
- Audit facts: last readiness decision, who cleared each review item, source timestamps, and invalidation events.

### Decisions

The policy answers one question: can front-desk staff complete the immediate customer interaction quickly and safely with only the expected interaction script?

The decision is not a generic `ready: bool`; it is a closed semantic outcome:

- `ReadyToCheckIn`: identity, reservation, eligibility, care notes, assignment, capacity/staffing, and payment/package preconditions are resolved for the requested service.
- `ReadyWithExpectedCollection`: check-in may proceed after front desk collects a predictable, non-policy-blocking item such as an already-requested document copy or customer signature, with a typed collection task.
- `NeedsFrontDeskCollection`: missing data/document/payment information blocks fast throughput but is collectable by front desk without manager/pet-care judgment.
- `NeedsCareTeamReview`: care note, temperament, assignment, incident, medical, allergy, medication, or handling uncertainty requires kennel/play staff or lead review before check-in proceeds.
- `NeedsManagerReview`: hard stop, override, capacity/ratio exception, refund/waiver/discount, sensitive incident message, or reinstatement/suspension decision requires manager approval.
- `WaitlistOrCapacityHold`: deterministic availability/capacity/staffing is not ready; front desk may explain waitlist/hold status using approved wording but may not promise availability.
- `BlockedForSafetyOrPolicy`: current facts make the requested daycare path ineligible; front desk must route to approved alternatives or manager review.
- `ReadyToCheckOut`: care notes, incident disclosures, belongings, checkout charges, package consumption, and approved customer communication are ready for departure.
- `CheckoutNeedsReview`: unresolved charge/package/refund/incident/care-note/customer-message issue blocks fast checkout.

### Outputs

- `operations::daycare::front_desk::ReadinessDecision` with typed reasons, review gates, expiration/invalidation basis, and source snapshot IDs.
- `operations::StaffTask` values for `CheckInPrep`, `CheckOutPrep`, `DocumentReview`, `PlaygroupAssessment`, `IncidentFollowUp`, `CustomerFollowUp`, or a proposed `CareHandoffReview` if later code adds a daycare-specific task kind.
- `workflow::WorkflowEvent` records such as `DaycareCheckInReadinessEvaluated`, `DaycareReadinessInvalidated`, `DaycareFastLaneOpened`, `DaycareReviewLaneOpened`, and `DaycareCheckoutReadinessEvaluated`.
- Front-desk queue views: fast lane, collection lane, care-team review lane, manager review lane, waitlist/capacity lane, and checkout review lane.
- Agent prompt packets for approved draft/recommend workflows: readiness summary, missing-requirements summary, internal task draft, manager daily brief item, or customer follow-up draft.
- Audit entries showing who/what decided readiness, which facts were read, which gates were cleared, and which customer-facing action was only drafted.

### Success state

A successful fast-throughput path means:

- The front desk can greet the customer, identify the reservation/pet, confirm approved service wording, complete any expected collection, and hand off the pet or complete checkout without hunting through notes or making policy judgments.
- Safety-critical facts are either resolved or routed to the right human owner before the customer reaches the counter.
- Group-play, day-boarding, hybrid, and cat-care paths remain distinct; a pet may be fast-lane-ready for individual day boarding while blocked from dog group play.
- Any customer-facing message, payment action, discount/waiver/refund, or eligibility override has the correct approval trail.
- Queue-time improvements are explainable from typed readiness states, not opaque automation.

### Failure and exception states

- Missing, stale, ambiguous, or conflicting vaccine/temperament/spay-neuter/care/payment facts produce review or collection states, never accidental `ReadyToCheckIn`.
- A same-day walk-in or requested service change creates a fresh readiness context; it cannot reuse a stale pre-arrival readiness decision for a different service variant/care mode.
- Capacity or staff-ratio changes invalidate group-play readiness and may move the pet to waitlist, individual day boarding, or manager review depending on policy.
- A suspending incident, medical/care concern, or sensitive behavior note invalidates fast-lane readiness until the proper owner clears it.
- Payment/package exceptions can block checkout or require manager approval, but the agent must not capture payment, issue refunds, apply discounts, or enroll packages autonomously.
- If the source system is unavailable, the workflow may present a `SourceUnavailableNeedsManualReview` reason and create an internal task; it must not infer readiness from cached data unless the cache has an explicit freshness/invalidation contract.

## 2. Domain types to add or refine

Keep the public path under `operations::daycare::front_desk` when the concept is specific to daycare counter throughput. Re-export only the central entry points from `operations::daycare` if call sites benefit, e.g. `operations::daycare::FrontDeskThroughputPolicy`.

### Entities and aggregate roots

- `operations::daycare::front_desk::ReadinessContext`
  - Aggregate input for one pet/reservation/location/service/date interaction.
  - Invariants: one customer, one pet, one location, one requested `ServiceVariant`, one derived `CareMode`, and one interaction phase (`CheckIn` or `CheckOut`). It carries typed snapshots, not raw provider notes.
- `operations::daycare::front_desk::ReadinessDecision`
  - Aggregate output of the policy.
  - Invariants: exactly one `ReadinessOutcome`, non-empty reasons for every non-ready outcome, required review gates attached to every human-gated outcome, and an expiration/invalidation basis.
- `operations::daycare::front_desk::QueueTicket`
  - Front-desk queue item for a reservation interaction.
  - Invariants: references a current readiness decision, has a semantic lane, and preserves ordering basis without promising service availability.
- `operations::daycare::front_desk::AuditEntry`
  - Immutable record of decision source, actor, policy version, and cleared/uncleared gates.
  - Invariants: no raw sensitive staff note text in debug/customer-safe rendering; sensitive rationale uses redacted semantic references.

### Value objects and newtypes

- `operations::daycare::front_desk::QueuePosition(u16)`
  - Non-zero when displayed as customer-facing sequence; internal zero-based indexes stay in adapters.
- `operations::daycare::front_desk::ExpectedWaitMinutes(u16)`
  - Bounded estimate with explicit `Unknown` outcome elsewhere; do not represent unknown as zero.
- `operations::daycare::front_desk::ReadinessSnapshotId`
  - Non-empty source-neutral ID for the fact snapshot used by the decision.
- `operations::daycare::front_desk::ReadinessPolicyVersion`
  - Non-empty semantic version/source label for audit and regression tests.
- `operations::daycare::front_desk::CustomerSafeSummary`
  - Sanitized/approved wording for front-desk scripts; cannot contain unapproved incident/health/sensitive behavior detail.
- `operations::daycare::front_desk::InternalReadinessNote`
  - Staff-only rationale value with redacted debug; source staff notes stay in the owning care/temperament/incident modules.
- `operations::daycare::front_desk::InteractionDeadline`
  - Time by which a review/collection item must be resolved for fast-lane status.

### Enums and invariant-bearing concepts

```rust
operations::daycare::front_desk::InteractionPhase::{
    CheckIn,
    CheckOut,
}

operations::daycare::front_desk::ReadinessOutcome::{
    ReadyToCheckIn { lane: ThroughputLane },
    ReadyWithExpectedCollection { collection: CollectionRequirement, lane: ThroughputLane },
    NeedsFrontDeskCollection { requirements: Vec<CollectionRequirement> },
    NeedsCareTeamReview { reasons: Vec<CareReviewReason> },
    NeedsManagerReview { reasons: Vec<ManagerReviewReason>, gates: Vec<policy::ReviewGate> },
    WaitlistOrCapacityHold { reason: CapacityHoldReason },
    BlockedForSafetyOrPolicy { reasons: Vec<SafetyOrPolicyBlock> },
    ReadyToCheckOut { lane: ThroughputLane },
    CheckoutNeedsReview { reasons: Vec<CheckoutReviewReason> },
    SourceUnavailableNeedsManualReview { source: SourceSystem, stale_snapshot: Option<ReadinessSnapshotId> },
}

operations::daycare::front_desk::ThroughputLane::{
    FastLane,
    ExpectedCollection,
    FrontDeskCollection,
    CareTeamReview,
    ManagerReview,
    WaitlistOrCapacity,
    CheckoutReview,
}
```

Additional enum families:

- `CollectionRequirement`: vaccine proof copy, customer signature, package selection confirmation, contact preference confirmation, pickup authorization confirmation, belongings label, payment method confirmation.
- `CareReviewReason`: medication/allergy/medical note requires review, feeding note conflict, temperament evidence missing/stale, playgroup assignment missing, incident pending follow-up, handling note ambiguous.
- `ManagerReviewReason`: capacity/ratio override, policy hard stop, group-play reinstatement/suspension, refund/waiver/discount/package correction, sensitive incident/customer message approval.
- `CapacityHoldReason`: full playgroup, insufficient staff coverage, room/rest lane unavailable, hybrid service missing one lane, source availability unknown.
- `SafetyOrPolicyBlock`: vaccine not current, age below minimum, group-play ineligible, suspending incident, service unavailable for species/care mode, medical hard stop.
- `CheckoutReviewReason`: unresolved charge, package balance conflict, refund/credit exception, incident/customer notice gate, belongings/handoff missing, daily-care update unapproved.
- `ReadinessInvalidationReason`: reservation changed, service variant changed, care note changed, incident created/updated, vaccine proof changed, spay/neuter fact changed, temperament evidence changed, capacity/staffing changed, payment/package state changed, policy version changed, source snapshot expired.

### Builders and policies

- `ReadinessContext::builder()` requires customer, pet, location, reservation, service variant, care mode, phase, policy snapshot, eligibility snapshot, capacity/staff snapshot, care snapshot, payment/package snapshot, and source freshness. Optional notes default to absent; missing required evidence becomes typed review reason.
- `ReadinessDecision::builder()` should be private or policy-owned so call sites cannot create `ReadyToCheckIn` without satisfying invariants.
- `QueueTicket::builder()` requires a `ReadinessDecision` and derives lane from the outcome; callers should not supply arbitrary lane strings.
- `FrontDeskThroughputPolicy::evaluate(context) -> ReadinessDecision` owns readiness semantics.
- `ReadinessInvalidationPolicy::invalidate(previous, event) -> ReadinessInvalidation` owns stale-decision detection.

## 3. Relationship map between types

### Entities / aggregates

- `ReadinessContext` references `entities::CustomerId`, `entities::PetId`, `entities::LocationId`, and `reservation::Id`/`entities::ReservationId` but does not own those identities.
- `ReadinessDecision` composes `GroupPlayEligibilityDecision`, `StaffCoverageDecision`, `PlaygroupAssignment`, payment/package state, care-review state, and queue lane.
- `QueueTicket` belongs to the operating-day front-desk queue and is derived from a readiness decision.
- `AuditEntry` records the decision, actor, policy version, gates, and source snapshot IDs.

### Value objects

- `ReadinessSnapshotId`, `ReadinessPolicyVersion`, `QueuePosition`, `ExpectedWaitMinutes`, `CustomerSafeSummary`, `InternalReadinessNote`, and `InteractionDeadline` are daycare front-desk values.
- Existing `operations::SnapshotId`, `operations::OperationalObservation`, and `operations::OperationalRecommendation` can be reused for daily briefs and manager-level summaries, but the readiness core should keep specific reasons under `operations::daycare::front_desk`.

### Policies

- `FrontDeskThroughputPolicy` is the main readiness owner.
- `ReadinessInvalidationPolicy` prevents stale green-light decisions.
- `GroupPlayEligibilityPolicy` remains daycare eligibility owner; throughput consumes its decision instead of duplicating group-play rules.
- `StaffCoveragePolicy` owns ratio/capacity/staffing decision; throughput only maps insufficient/unknown coverage into lanes and tasks.
- `PaymentReadinessPolicy` or a future `operations::daycare::front_desk::CheckoutPolicy` should classify package/payment readiness without executing payment.
- `CustomerCommunicationPolicy`/`policy::ReviewGate` owns customer-facing boundaries; throughput can require gates, not bypass them.

### Repositories / stores

- `operations::daycare::front_desk::ReadinessRepository`: persists readiness decisions, queue tickets, invalidations, and audit entries.
- `operations::daycare::PolicyRepository`: loads daycare contract, policy version, local overrides, and review-gate rules.
- `operations::daycare::AttendanceRepository`: supplies reservation/attendance/recurrence facts and records check-in/check-out facts after staff action.
- `operations::daycare::RosterRepository`: supplies playgroup/room/yard capacity, staff assignments, and current queue/roster state.
- `operations::daycare::EligibilityRepository`: supplies current eligibility decisions and evidence snapshots.
- `operations::daycare::IncidentRepository`: supplies unresolved restrictions and stores follow-up dispositions.
- Boundary stores in `tools`, `reservation`, `payment`, and provider adapters convert raw PMS/portal/payment values into the semantic snapshots consumed by readiness.

### Workflow events

- `DaycareReadinessEvaluationRequested`
- `DaycareCheckInReadinessEvaluated`
- `DaycareCheckoutReadinessEvaluated`
- `DaycareReadinessInvalidated`
- `DaycareFastLaneOpened`
- `DaycareReviewLaneOpened`
- `DaycareCollectionTaskCreated`
- `DaycareManagerReviewRequested`
- `DaycareCustomerMessageDrafted`

### Staff tasks

Existing task kinds to reuse:

- `operations::StaffTaskKind::CheckInPrep { reservation_id }`
- `operations::StaffTaskKind::CheckOutPrep { reservation_id }`
- `operations::StaffTaskKind::DocumentReview { pet_id }`
- `operations::StaffTaskKind::PlaygroupAssessment { pet_id }`
- `operations::StaffTaskKind::IncidentFollowUp { pet_id }`
- `operations::StaffTaskKind::CustomerFollowUp { customer_id, reason }`

Potential later refinements if tests prove the existing task kinds are too broad:

- `operations::StaffTaskKind::CareHandoffReview { reservation_id, pet_id }`
- `operations::StaffTaskKind::PackageBalanceReview { customer_id, pet_id }`
- `operations::StaffTaskKind::FrontDeskCollection { reservation_id }`

### Agent specs and tools

- Existing `booking-triage`: evaluates missing info, availability, eligibility, and readiness; can draft but not confirm.
- Existing `manager-daily-brief`: summarizes unresolved lanes and throughput bottlenecks.
- Existing `daily-care-update`: drafts approved checkout/daily update copy when staff notes are available.
- Existing `incident-escalation`: summarizes sensitive incidents and routes gates.
- Proposed `front-desk-readiness` agent spec: reads readiness facts, produces typed readiness summary, proposes staff tasks, and drafts customer-safe collection scripts. Allowed tools: `reservation-read`, `policy-read`, `availability-read`, `care-note-read`, `payment-read`, `task-create`, `draft-message`. Forbidden actions: confirm booking, override policy, change assignment, capture/refund payment, apply discount/waiver, enroll package, send customer message, hide safety facts.

## 4. Interaction contract

Use contracts that put behavior on truthful owners.

```rust
pub trait FrontDeskThroughputPolicy {
    fn evaluate(
        &self,
        context: operations::daycare::front_desk::ReadinessContext,
    ) -> operations::daycare::front_desk::Result<operations::daycare::front_desk::ReadinessDecision>;
}

impl operations::daycare::front_desk::ReadinessDecision {
    pub fn outcome(&self) -> &operations::daycare::front_desk::ReadinessOutcome;
    pub fn lane(&self) -> operations::daycare::front_desk::ThroughputLane;
    pub fn requires_human_review(&self) -> bool;
    pub fn required_review_gates(&self) -> &[policy::ReviewGate];
    pub fn is_fast_lane_ready_for(&self, phase: InteractionPhase) -> bool;
    pub fn invalidates_on(&self) -> &[ReadinessInvalidationReason];
}

pub trait ReadinessInvalidationPolicy {
    fn invalidate_for(
        &self,
        previous: &operations::daycare::front_desk::ReadinessDecision,
        event: operations::daycare::front_desk::ReadinessInvalidationEvent,
    ) -> Option<operations::daycare::front_desk::ReadinessInvalidation>;
}
```

Repository contracts:

```rust
#[async_trait]
pub trait operations::daycare::front_desk::ReadinessRepository {
    async fn load_current(
        &self,
        reservation: reservation::Id,
        phase: InteractionPhase,
    ) -> front_desk::Result<Option<ReadinessDecision>>;

    async fn save_decision(
        &self,
        decision: ReadinessDecision,
        audit: AuditEntry,
    ) -> front_desk::Result<()>;

    async fn enqueue(
        &self,
        ticket: QueueTicket,
    ) -> front_desk::Result<()>;

    async fn append_invalidation(
        &self,
        invalidation: ReadinessInvalidation,
        audit: AuditEntry,
    ) -> front_desk::Result<()>;
}

#[async_trait]
pub trait operations::daycare::front_desk::ReadinessContextRepository {
    async fn build_context(
        &self,
        request: ReadinessContextRequest,
    ) -> front_desk::Result<ReadinessContext>;
}
```

Domain service contracts:

```rust
impl operations::daycare::FrontDeskThroughputService {
    pub async fn evaluate_and_enqueue(
        &self,
        request: front_desk::ReadinessContextRequest,
    ) -> operations::daycare::Result<front_desk::ReadinessDecision>;

    pub async fn handle_invalidation(
        &self,
        event: front_desk::ReadinessInvalidationEvent,
    ) -> operations::daycare::Result<Vec<front_desk::QueueTicket>>;
}
```

Behavior ownership rules:

- Eligibility rules live in `GroupPlayEligibilityPolicy`; throughput only consumes decisions and maps them to lanes/tasks.
- Staff ratio/capacity rules live in `StaffCoveragePolicy`/`RosterRepository`; throughput does not recalculate with naked integers.
- Care/medical/allergy/medication note interpretation lives in `care` and staff review gates; throughput does not diagnose or suppress.
- Payment execution lives in payment tools/gateways; checkout readiness classifies payment/package state and required approvals.
- Customer-message approval lives in `policy::ReviewGate` and workflow/customer communication modules; throughput may draft collection scripts but not send them.
- Provider/PMS lookup and raw field parsing live in adapters; the domain contract sees semantic snapshots and typed source-freshness errors.

## 5. Review and approval contract

### Automation level

- Safe to automate: read-only context assembly, deterministic readiness evaluation, queue lane recommendation, stale-decision invalidation, internal task draft/create where task creation is allowed, manager brief summary, customer-message draft generation.
- Draft/internal-task only: missing requirement collection script, package opportunity script, customer follow-up draft, review packet summary.
- Manager approval required: capacity/ratio override, group-play suspension/reinstatement, hard-stop override, sensitive incident/health/behavior owner message, refund/waiver/discount/package correction, staff schedule change.
- Never automate: final live check-in when a hard stop is unresolved, payment capture/refund/discount/enrollment, hiding incident/safety facts, medical/behavior diagnosis, customer-facing send without review gate, policy override.

### Review gates

Map readiness outcomes to gates explicitly:

- `NeedsFrontDeskCollection`: front-desk staff may resolve when the requirement is clerical and already policy-permitted.
- `NeedsCareTeamReview`: requires lead/kennel/play staff completion evidence; medical document ambiguity maps to `policy::ReviewGate::MedicalDocumentReview`.
- `NeedsManagerReview`: requires `policy::ReviewGate::ManagerApproval` and any more specific gate such as `RefundOrDepositException` or `CustomerMessageApproval`.
- `CheckoutNeedsReview` with incident/health/sensitive behavior output requires both manager and customer-message approval before owner-facing communication.
- `ReadyWithExpectedCollection` must name what may be collected and what cannot be changed at the counter without reevaluation.

### Audit trail

Each decision/audit entry should capture:

- Context request ID, reservation/pet/customer/location IDs, phase, service variant, care mode, policy version, and source snapshot IDs.
- Decision outcome, reasons, lane, gates, and expiration/invalidation basis.
- Actor type (`SystemPolicy`, `AgentDraft`, `FrontDeskStaff`, `CareTeamStaff`, `Manager`) and staff ID when human-cleared.
- Before/after state for invalidations and manual overrides.
- Customer-facing artifact IDs for drafts/approved sends, never raw unapproved sensitive note text.
- Payment/package action references only after approved execution by the owning payment workflow.

### Customer/member-facing boundaries

- A readiness summary is staff-facing by default.
- Customer-safe summaries must be separately typed and approved before display/send.
- Agents can draft: "We still need updated vaccine proof before daycare check-in" or "A team member will review today's daycare availability." They must not draft unsupported promises such as "Your pet is approved for group play" unless the policy decision and approval trail support it.
- Incident, health, allergy, medication, and sensitive behavior details require configured review before owner-facing communication.
- Revenue/package suggestions are recommendations; no automatic enrollment, charge, discount, waiver, refund, or package correction.

## 6. Test contracts

These named semantic tests should pass when the later serialized Rust code card implements the model:

1. `front_desk_fast_lane_requires_resolved_daycare_readiness`
   - A check-in decision becomes `ReadyToCheckIn` only when reservation identity, service variant/care mode, eligibility, care notes, assignment, capacity/staffing, and payment/package preconditions are resolved.

2. `missing_vaccine_proof_routes_to_front_desk_collection_or_document_review`
   - If the policy allows front-desk collection of an expected document, the outcome is `ReadyWithExpectedCollection`; ambiguous/stale proof routes to `NeedsFrontDeskCollection` or `DocumentReview`, not fast-lane ready.

3. `unknown_temperament_blocks_group_play_fast_lane_without_blocking_individual_day_boarding_by_default`
   - Missing group-play temperament evidence prevents `CareMode::DogGroupPlay` readiness but can route a safe individual day-boarding request through its own care/capacity checks.

4. `capacity_or_staffing_change_invalidates_prior_fast_lane_decision`
   - A current `ReadyToCheckIn` decision is invalidated when roster capacity or staff coverage changes for the assigned care lane.

5. `same_day_service_change_rebuilds_readiness_context`
   - Changing from half-day play to Day Play Plus Room or day boarding creates a new readiness context and does not reuse stale readiness from the previous service variant.

6. `care_note_uncertainty_routes_to_care_team_review`
   - Medication, allergy, feeding, medical, or ambiguous handling notes produce `NeedsCareTeamReview` with typed reasons and staff task output.

7. `incident_restriction_requires_manager_or_incident_review_before_fast_lane`
   - A suspending or unresolved incident prevents fast-lane group-play check-in and emits the correct review gate/task.

8. `checkout_ready_requires_package_payment_and_customer_message_gates`
   - `ReadyToCheckOut` requires package visit consumption/payment state, unresolved charges, belongings/handoff, and approved customer-facing notes to be resolved.

9. `refund_waiver_discount_and_package_correction_never_execute_from_readiness_policy`
   - Readiness can classify and require approval for money/package exceptions but cannot call payment execution or mutate package balance by itself.

10. `customer_safe_summary_redacts_sensitive_unapproved_notes`
    - Staff-only rationale and sensitive care/incident/behavior facts do not appear in `CustomerSafeSummary` unless the required review gates are satisfied.

11. `queue_ticket_lane_is_derived_from_readiness_outcome`
    - A queue ticket cannot claim `FastLane` when the decision outcome is review, waitlist, collection, blocked, or checkout review.

12. `source_unavailable_uses_manual_review_not_cached_green_light_without_freshness_contract`
    - Source-system failure produces `SourceUnavailableNeedsManualReview` unless the cached snapshot explicitly satisfies freshness and invalidation requirements.

13. `front_desk_agent_spec_forbids_booking_payment_policy_override_and_message_send`
    - The proposed `front-desk-readiness` agent spec exposes read/draft/task tools only and includes forbidden actions for booking confirmation, payment/refund/discount/enrollment, policy override, and unapproved sends.

14. `readiness_audit_records_policy_version_snapshots_actor_and_review_gates`
    - Every saved decision carries policy version, source snapshot IDs, actor, lane, reasons, gates, and invalidation basis.

15. `readiness_outcomes_have_non_empty_reasons_when_not_ready`
    - Non-ready outcomes cannot be constructed with empty reason vectors or missing review gates where human approval is required.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add `operations::daycare::front_desk` submodule or split `operations/daycare/front_desk.rs` if the module is later decomposed.
  - Add `FrontDeskThroughputPolicy`, `ReadinessContext`, `ReadinessDecision`, `ReadinessOutcome`, `ThroughputLane`, invalidation types, queue ticket, audit values, and module-local `Error`/`Result`.
  - Potentially re-export `FrontDeskThroughputPolicy` and `FrontDeskReadiness` from `operations::daycare` for ergonomic call sites.
- `domain/src/policy.rs`
  - Reuse existing `ReviewGate`; add variants only if current gates cannot distinguish front-desk collection from manager/customer/payment gates.
- `domain/src/agents.rs`
  - Add a baseline `front-desk-readiness` spec or extend `booking-triage` if product chooses not to add a separate agent.
- `domain/src/tools.rs`
  - Add read-only readiness input tools or provider adapters if current availability/reservation/payment/care reads cannot assemble `ReadinessContext` semantically.
- `domain/src/workflow.rs` or existing workflow event modules
  - Add readiness evaluation/invalidated/queue/task events if events are modeled as typed enums.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Extend core contract tests for `FrontDeskThroughputPolicy` and readiness invariants.
- `domain/tests/domain_quality_patterns.rs`
  - Add call-site/path tests ensuring daycare front-desk paths preserve semantic module context.
- New tests such as `domain/tests/petsuites_daycare_front_desk_throughput.rs`
  - House the semantic test contracts above.

### Migration and refactor risks

- Avoid turning `FrontDeskReadiness` into a boolean or a raw status string. It should be an enum-centered model with typed reasons and gates.
- Do not duplicate group-play eligibility, vaccine, spay/neuter, incident, care, payment, or capacity logic inside the throughput policy. Consume decisions from truthful owners.
- Be careful with the existing top-level `operations::DaycareFormat` and `operations::DaycareEligibilityRule`; later refactors should move/re-export them under `operations::daycare` without creating parallel public surfaces.
- `operations::StaffTaskKind` may be broad enough for initial implementation. Add daycare-specific task kinds only if tests show the existing values lose operational meaning at call sites.
- Source adapters may be tempted to pass raw notes and provider codes into the domain. Convert to semantic snapshots and source-freshness values at the boundary.
- Fast throughput should not become silent auto-check-in. The policy can say "ready"; the staff/tool boundary still owns live status mutation.
- Checkout readiness must not execute payment/package mutations. It only classifies readiness and required approval.
- Customer-facing summaries need redacted types and review gates; do not reuse internal rationale strings.
- Cached readiness must have explicit invalidation/freshness semantics or it becomes unsafe under same-day capacity, staffing, incident, and care-note changes.

### Dependencies on other daycare implications

- Depends on the core daycare service map for `ServiceVariant`, `CareMode`, `GroupPlayEligibilityDecision`, `StaffCoverageDecision`, `PlaygroupAssignment`, package policy, and repository boundaries.
- Should align with any later daycare implication for group-play eligibility, recurring attendance, incidents, package/membership opportunities, and daily care updates.
- If another card defines `reservation::daycare` contracts, use those as the source of reservation identity/window facts while keeping front-desk readiness under `operations::daycare::front_desk`.
- If a future payment/package card defines package ledger semantics, checkout readiness should consume ledger decisions rather than define its own balance model.
- If a future agent-governance card defines central approval/audit abstractions, map `AuditEntry` and readiness gates to those shared workflow types rather than duplicating audit storage.

Implementation stance: create the semantic contract and tests before any live integration. The first code increment should prove construction invariants, outcome/gate mapping, stale-decision invalidation, and customer-safe redaction with deterministic unit tests before provider adapters or agent prompts are wired in.
