# Boarding implication 01: Capacity management

Purpose: define the Boarding capacity-management operational implication for the PetSuites AI-operations foundation. This is a modeling/spec artifact for later Rust work. It does not authorize live booking, room assignment, waitlist movement, customer messaging, payment, or member-facing action.

Sources read:

- `docs/domain/petsuites/boarding/service-domain-map.md`
- `domain/src/operations.rs`
- `domain/tests/petsuites_core_service_contracts.rs`

Assumptions:

- Capacity is location-scoped and local-policy-scoped. Room counts, room labels, holiday rules, holds, and overbooking permissions are provider/location data, not hard-coded constants.
- A boarding reservation may cover multiple pets, but the capacity decision must be per pet/accommodation segment before it can be summarized at reservation level.
- Existing `operations::boarding::RoomInventory`, `CapacityPlan`, and `RoomAvailability` are the compatibility bridge. They should not be deleted abruptly; later code should refine them into segmented capacity types while preserving storage roundtrips.
- Safe default: agents may surface capacity risk, draft internal tasks, and recommend waitlist/manager-review outcomes; they may not confirm availability, allocate rooms, override holds, or promise capacity to customers without typed staff/manager approval.

## 1. Operational story

### Trigger

Capacity management starts whenever the system needs to know whether a proposed, changed, or already-booked Boarding stay can be accommodated safely at a PetSuites location.

Common triggers:

- New boarding quote, website reservation request, front-desk intake, phone/call-center lead, or portal booking attempt.
- Reservation modification: date extension, early checkout, added pet, accommodation upgrade, downgrade, or add-on that consumes specialized capacity.
- Pre-arrival audit for upcoming stays, especially before holiday/peak periods.
- Daily/shift operations review: occupancy snapshot, arrivals/departures, room turnovers, room holds, maintenance closures, or staffing risk.
- Manager action: open/close waitlist, release held rooms, approve an exception, or reconcile provider data.
- Agent workflow: demand forecast, capacity alert, boarding daily brief, or missing-data detection.

### Actors

- Customer/member: requests or changes a stay, but does not own capacity decisions.
- Front desk / call-center staff: captures request, explains reviewed outcomes, and can apply approved staff-level decisions.
- Kennel technician / lead staff: reports room readiness, blocked rooms, turnover status, and pet-specific accommodation constraints.
- Manager: owns capacity exceptions, hold release, over-capacity review, holiday/peak policy exceptions, and denial/waitlist decisions when customer impact or safety risk is material.
- Domain policy: deterministically evaluates the request against inventory, reservations, holds, season policy, and species/accommodation constraints.
- Agent: reads capacity evidence, summarizes risk, drafts internal tasks/recommendations, and routes review; never directly confirms or changes live capacity.
- Provider/storage adapters: supply reservation, inventory, room, and hold snapshots; execute approved drafts only after review gates clear.

### Inputs

Required inputs for a truthful capacity decision:

- `entities::LocationId` and active `operations::boarding::Contract` / contract version.
- `operations::boarding::StayRequest` or reservation projection: customer, pets, date range, requested accommodation/flexibility, requested add-ons/upgrades, and source channel.
- Pet facts: `entities::PetId`, `entities::Species`, size/weight when available, temperament/care flags that may restrict room or play accommodation.
- `operations::boarding::stay::DateRange` and derived `stay::Nights`.
- Requested `operations::boarding::accommodation::Kind` per pet, or an explicit `accommodation::Flexibility` when the customer accepts alternatives.
- Capacity evidence: segmented inventory, booked counts, operational holds, maintenance closures, turnover/readiness state, waitlist counts, and provider timestamps.
- Season/holiday policy: named demand period, minimum stays, blackout/hold rules, deposit/cancellation interaction, and review thresholds.
- Existing reservation state: tentative holds, confirmed stays, checked-in stays, cancellations pending release, and overstay/early-arrival signals.
- Staff context where available: staffing level, room-cleaning backlog, unresolved incidents, or open manager holds.

### Decisions

Capacity management must answer typed questions instead of returning a loose `available: bool`:

1. Is the requested date range valid and policy-observable for the location timezone?
2. Which accommodation segment(s) are eligible for each pet?
3. Does the segment have enough available, cleanable, non-held capacity for each night?
4. Does a peak/holiday/local-event policy modify minimum stay, hold, waitlist, deposit, or manager-review rules?
5. Can the system produce an internal assignment candidate, or only an availability posture?
6. Should the request be available, limited, waitlisted, denied, or routed to manager review?
7. Which staff tasks or workflow recommendations are needed before any customer/member-facing action?
8. Which evidence and policy version must be retained for audit and later dispute resolution?

### Outputs

Primary domain output:

- `operations::boarding::capacity::Decision`
  - `Available { posture, assignable, evidence, required_reviews }`
  - `Limited { remaining, posture, required_reviews, evidence }`
  - `Waitlist { reason, waitlist_position_hint, evidence }`
  - `Deny { reason, evidence }`
  - `ManagerReview { reason, evidence, recommended_tasks }`

Supporting outputs:

- `operations::boarding::capacity::Snapshot` for manager/agent/staff visibility.
- `operations::boarding::capacity::HoldDraft` or `AssignmentDraft` when staff-approved execution is possible.
- `operations::boarding::task::Kind` mapped into `operations::StaffTask` drafts: manager capacity review, room readiness check, turnover verification, accommodation mismatch review, waitlist follow-up, provider data reconciliation.
- `workflow::RecommendedAction` values for reviewable next steps.
- `workflow::WorkflowEvent` / audit trail entries recording request, evidence version, policy version, decision, actor, and approval state.
- Capacity risk items for `operations::ResortDailyBrief`, `OccupancySnapshot`, and `OperationsRisk`.

### Success state

A capacity-management run succeeds when:

- The decision is typed and explainable from immutable evidence snapshots.
- Accommodation eligibility is species-safe; cats cannot consume dog-suite capacity and dogs cannot consume cat-condo capacity.
- Every night in the stay has segment-level capacity evaluated, not just arrival-day capacity.
- Holiday/peak/local-event rules are applied before customer-facing confirmation.
- Holds, closures, and provider staleness are visible in the decision.
- Any member-facing or live-provider action is either blocked behind review or carries a typed approval record.
- Staff can see concrete tasks required to make the decision operable.

### Failure and exception states

- `InvalidStayRange`: checkout does not follow checkin, location timezone is missing when local dates matter, or derived nights is zero.
- `UnknownLocationContract`: no active Boarding contract/policy snapshot exists for the location.
- `UnsupportedAccommodation`: requested product is not in the location catalog.
- `SpeciesAccommodationMismatch`: pet species cannot use the requested segment.
- `CapacityDataUnavailable`: provider/storage cannot provide reservation or inventory evidence.
- `StaleCapacitySnapshot`: snapshot is older than the configured freshness threshold for booking decisions.
- `SegmentFull`: no eligible capacity for at least one required night.
- `RoomHeldOrClosed`: capacity exists in count form but is blocked by manager hold, maintenance, turnover, or quarantine-like operational closure.
- `PeakPolicyRequiresReview`: holiday/peak/local event policy requires manager review or manual release.
- `WaitlistOnly`: segment is configured to waitlist-only even if nominal capacity exists.
- `MultiPetSplitRequired`: pets in the same customer request cannot all be placed under the requested accommodation/flexibility.
- `ProviderConflict`: reservation provider state conflicts with local projections.
- `OverbookingRequiresManager`: any decision that would exceed configured capacity must be explicit manager review, never silent availability.

## 2. Domain types to add or refine

### Compatibility bridge to preserve

- `operations::boarding::Contract`
  - Keep as the service-line root contract.
  - Add capacity-policy references or child values in place rather than creating a parallel root type.
- `operations::boarding::CapacityPlan`
  - Current contract-level posture: `RoomInventory` + `RoomAvailability`.
  - Refine into a summary/compatibility projection over segmented capacity, not the final source of truth.
- `operations::boarding::RoomInventory`
  - Current positive scalar.
  - Re-export or replace with `capacity::RoomCount` only after serialization migration is explicit.
- `operations::boarding::RoomAvailability`
  - Current broad availability posture.
  - Refine into `capacity::Availability` with manager-hold/waitlist/closed reasons.

### Proposed semantic paths and invariants

- `operations::boarding::capacity::Inventory`
  - Segment-indexed room counts by `accommodation::Kind`.
  - Invariants: at least one segment; every configured segment has positive `RoomCount`; no duplicate segment keys.
- `operations::boarding::capacity::Segment`
  - A capacity-bearing location/accommodation partition.
  - Fields: `location::Id`, `accommodation::Kind`, optional `capacity::SegmentId`.
  - Invariant: segment kind must be supported by the location catalog.
- `operations::boarding::capacity::RoomCount`
  - Positive count for configured inventory.
  - Zero is invalid; closures/holds are separate state, not zero inventory.
- `operations::boarding::capacity::BookedCount`
  - Non-negative count of committed rooms/pets for a night/segment.
  - May be zero; must never be interpreted as total capacity.
- `operations::boarding::capacity::AvailableCount`
  - Non-negative count derived from inventory minus bookings/holds/closures.
  - If negative evidence appears, produce `OverCapacity` evidence rather than wrapping/saturating silently.
- `operations::boarding::capacity::OccupancyBasisPoints`
  - 0..=10_000 for normal occupancy. Overcapacity uses `OverCapacityBy`, not >10_000.
- `operations::boarding::capacity::OverCapacityBy`
  - Positive count representing a provider or operational exception already beyond capacity.
- `operations::boarding::capacity::Snapshot`
  - Immutable evidence for a location/date-range/segment evaluation.
  - Invariants: includes source timestamp, policy version, and one `NightlySegmentSnapshot` per evaluated night/segment.
- `operations::boarding::capacity::NightlySegmentSnapshot`
  - Per-night view: inventory, booked, holds, closures, waitlist posture, turnover/readiness flags.
  - Invariant: counts use typed scalars; source date is a local boarding night.
- `operations::boarding::capacity::Availability`
  - Enum: `Open`, `Limited { remaining: AvailableCount }`, `WaitlistOnly { reason }`, `Closed { reason }`, `ManagerHold { reason }`, `DataStale { observed_at }`.
- `operations::boarding::capacity::Decision`
  - Enum centered on operational outcome; includes evidence and review requirements.
- `operations::boarding::capacity::DenialReason`
  - Enum: `NoEligibleSegment`, `SegmentFull`, `ClosedForDates`, `UnsupportedAccommodation`, `SpeciesMismatch`, `MinimumStayNotMet`, `BlackoutPeriod`, `PolicyUnavailable`.
- `operations::boarding::capacity::WaitlistReason`
  - Enum: `SegmentFull`, `HolidayHold`, `ManagerHold`, `CustomerFlexibleDatesNeeded`, `StaffingOrTurnoverRisk`.
- `operations::boarding::capacity::ReviewReason`
  - Enum: `LimitedInventory`, `HolidayPeak`, `ManagerHoldRelease`, `ProviderDataConflict`, `OverCapacity`, `MultiPetSplit`, `RoomReadinessRisk`, `StaleProviderData`.
- `operations::boarding::capacity::EvidenceRef`
  - Opaque reference to provider/storage snapshot, event, or sync batch.
  - Invariant: non-empty and source-typed.
- `operations::boarding::capacity::FreshnessWindow`
  - Positive duration for maximum acceptable data age by action level.
- `operations::boarding::capacity::Hold`
  - Existing hold in evidence: manager hold, maintenance closure, quarantine/safety hold, turnover hold, strategic holiday hold.
- `operations::boarding::capacity::HoldDraft`
  - Proposed hold action; cannot execute without approval.
- `operations::boarding::accommodation::Kind`
  - `Dog(DogSuiteKind)` or `Cat(CatAccommodationKind)`; prevents dog/cat capacity leakage.
- `operations::boarding::accommodation::Flexibility`
  - `Exact(Kind)`, `AllowUpgrade { from, to }`, `AllowAnyEligibleDogSuite`, `AllowAnyEligibleCatAccommodation`, `StaffReviewRequired`.
- `operations::boarding::accommodation::AssignmentDraft`
  - Proposed room/segment assignment tied to evidence and approval state.
  - Invariant: draft is not a confirmed room assignment.
- `operations::boarding::room::RoomId`
  - Non-empty provider/location room identity.
- `operations::boarding::stay::DateRange`
  - Check-out follows check-in; derived `Nights` >= 1; local date basis is explicit.
- `operations::boarding::season::Period`
  - Named holiday/peak/local-event/blackout period with effective date range.
- `operations::boarding::capacity::Policy`
  - Deterministic evaluator. Owns capacity decision behavior.
- `operations::boarding::capacity::Repository`
  - Port for inventory, snapshots, holds, assignment drafts, and audit evidence.

## 3. Relationship map

### Entities

- `operations::boarding::StayRequest`
  - Request aggregate consumed by capacity policy.
  - References `entities::LocationId`, `entities::CustomerId`, pet IDs, date range, accommodation preference, add-ons, and source channel.
- `operations::boarding::StayPlan`
  - Internal accepted plan after capacity/care/deposit review. Capacity-management can produce the capacity-reviewed portion but not final confirmation alone.
- `entities::Reservation`
  - Cross-service lifecycle aggregate. Capacity should recommend reservation updates through workflow/tool drafts rather than direct mutation.
- `entities::Pet`
  - Supplies species and relevant profile facts used to choose eligible accommodation segments.
- `entities::Location`
  - Supplies location policy refs and capabilities.

### Value objects

- `capacity::Inventory`, `Segment`, `RoomCount`, `BookedCount`, `AvailableCount`, `OccupancyBasisPoints`, `Snapshot`, `NightlySegmentSnapshot`, `EvidenceRef`.
- `accommodation::Kind`, `DogSuiteKind`, `CatAccommodationKind`, `Flexibility`, `AssignmentDraft`.
- `stay::DateRange`, `stay::Nights`, `season::Period`.
- `capacity::FreshnessWindow`, `Hold`, `HoldDraft`.

### Policies

- `capacity::Policy`
  - Owns segment availability, per-night sufficiency, hold/closure interpretation, waitlist routing, and overbooking prohibition.
- `season::Policy`
  - Owns holiday/peak/local-event overlays that can change capacity posture.
- `stay::Policy`
  - Owns minimum stay and service-window constraints before capacity promises are made.
- `care::Policy`
  - Owns care facts that constrain accommodation or require review; capacity consumes the resulting constraints.
- `deposit::Policy`
  - May block confirmation after capacity is available; capacity should not decide money state.
- `agent::ApprovalPolicy`
  - Maps capacity decisions and proposed actions to automation/review levels.

### Repositories and stores

- `operations::boarding::capacity::Repository`
  - Read: inventory, nightly occupancy, holds, closures, waitlist posture, provider sync metadata.
  - Write: internal snapshots, hold drafts, assignment drafts, audit references. Live writes require approved tool drafts.
- `operations::boarding::reservation::Repository`
  - Query reservations by location/date/service/segment. Propose reservation updates through workflow/tool envelopes.
- `operations::boarding::room::Repository`
  - Read room metadata and readiness state; create assignment drafts.
- `operations::boarding::Repository`
  - Load Boarding contract/policy snapshots by location/version.
- `storage::operations::*`
  - Persistence adapters convert records to semantic domain values; provider JSON stays outside the core.

### Workflow events

- `workflow::WorkflowEvent::CapacityEvaluated` or equivalent future variant.
- `workflow::RecommendedAction::CreateStaffTask` for review/task drafts.
- `workflow::RecommendedAction::UpdateReservation` only as an approved/draft boundary action.
- Capacity audit event should include decision, policy version, evidence refs, actor, review gate, and whether customer-facing action is allowed.

### Staff tasks

Boarding-owned `task::Kind` should map into generic `operations::StaffTaskKind` without losing Boarding source context:

- `CapacityReview { stay_request_id, reason }`
- `WaitlistFollowUp { customer_id, date_range, reason }`
- `RoomReadinessCheck { segment, date }`
- `TurnoverRiskReview { segment, date }`
- `ManagerHoldReview { hold, requested_action }`
- `ProviderDataReconciliation { evidence_ref }`
- `MultiPetSplitReview { stay_request_id }`
- `AccommodationMismatchReview { pet_id, requested, eligible }`

### Agent specs and tools

- `agents::WorkflowAgent` may run `BoardingCapacityAlert`, `DemandForecasting`, `WebsiteReservationAssistant`, or `BoardingPreArrivalChecklistAutomation` specs.
- `agents::AgentPromptPacket` should carry only read-only evidence and approved policy context.
- `tools::AvailabilityRequest/Result` can be an external execution/query boundary, but must promote raw responses into `capacity::Snapshot` before domain evaluation.
- `tools::ReservationUpdateDraft`, room-assignment adapters, waitlist adapters, and messaging adapters must require approval tokens/review gates before live action.

## 4. Interaction contract

Rust-like pseudo-signatures; exact types can be adjusted during implementation, but ownership should stay truthful.

```rust
impl operations::boarding::capacity::Policy {
    pub fn evaluate(
        &self,
        request: &operations::boarding::StayRequest,
        snapshot: &operations::boarding::capacity::Snapshot,
        season: &operations::boarding::season::Policy,
        stay_policy: &operations::boarding::stay::Policy,
        care_constraints: &operations::boarding::care::AccommodationConstraints,
    ) -> operations::boarding::capacity::Result<operations::boarding::capacity::Decision>;
}
```

Behavior belongs on `capacity::Policy` because capacity rules are an explicit policy surface. `StayRequest` should not decide its own availability, and a generic helper should not own overbooking/hold/waitlist semantics.

```rust
impl operations::boarding::capacity::Snapshot {
    pub fn availability_for(
        &self,
        segment: operations::boarding::capacity::Segment,
        dates: operations::boarding::stay::DateRange,
    ) -> operations::boarding::capacity::Result<operations::boarding::capacity::SegmentAvailability>;

    pub fn assert_fresh_for(
        &self,
        action: operations::boarding::capacity::ActionSensitivity,
        now: time::OffsetDateTime,
    ) -> operations::boarding::capacity::Result<()>;
}
```

`Snapshot` can answer evidence-local questions and freshness checks. It should not apply business exceptions or manager policy.

```rust
trait operations::boarding::capacity::Repository {
    fn load_snapshot(
        &self,
        location: entities::LocationId,
        dates: operations::boarding::stay::DateRange,
        segments: operations::boarding::capacity::SegmentQuery,
    ) -> operations::boarding::capacity::Result<operations::boarding::capacity::Snapshot>;

    fn persist_decision_audit(
        &self,
        audit: operations::boarding::capacity::DecisionAudit,
    ) -> operations::boarding::capacity::Result<operations::boarding::capacity::AuditId>;

    fn draft_hold(
        &self,
        hold: operations::boarding::capacity::HoldDraft,
    ) -> operations::boarding::capacity::Result<workflow::RecommendedAction>;

    fn draft_assignment(
        &self,
        assignment: operations::boarding::accommodation::AssignmentDraft,
    ) -> operations::boarding::capacity::Result<workflow::RecommendedAction>;
}
```

Repository writes create drafts/audit, not live capacity mutation, unless a separate tool adapter receives an approval-cleared command.

```rust
impl operations::boarding::accommodation::Kind {
    pub fn eligible_for_species(
        &self,
        species: entities::Species,
    ) -> operations::boarding::accommodation::Eligibility;
}
```

Accommodation owns species compatibility because the invariant is about accommodation semantics, not provider storage.

```rust
impl operations::boarding::capacity::Decision {
    pub fn automation_level(
        &self,
        approval: &operations::boarding::agent::ApprovalPolicy,
    ) -> policy::AutomationLevel;

    pub fn required_review_gate(
        &self,
        approval: &operations::boarding::agent::ApprovalPolicy,
    ) -> Option<policy::ReviewGate>;

    pub fn to_staff_tasks(
        &self,
        request: &operations::boarding::StayRequest,
    ) -> Vec<operations::boarding::task::Kind>;
}
```

Decision can expose its implications, but the deterministic mapping to automation boundaries belongs in `agent::ApprovalPolicy` if action sensitivity varies by policy version/location.

```rust
impl operations::boarding::workflow::Planner {
    pub fn plan_capacity_followups(
        &self,
        request: &operations::boarding::StayRequest,
        decision: &operations::boarding::capacity::Decision,
    ) -> Vec<workflow::RecommendedAction>;
}
```

Workflow planning owns the conversion from domain decision to operational tasks/actions. Capacity policy should not know about every tool implementation.

```rust
impl operations::boarding::Contract {
    pub fn capacity_policy_ref(&self) -> operations::boarding::capacity::PolicyRef;
    pub fn default_capacity_summary(&self) -> operations::boarding::CapacityPlan;
}
```

`Contract` remains the root service-line contract and should expose capacity policy/configuration without becoming the evaluator.

## 5. Review and approval contract

### Automation level

Safe internal automation:

- Read provider/storage capacity snapshots.
- Build `capacity::Snapshot` and `capacity::Decision` from deterministic policy.
- Detect risk: full segment, stale snapshot, conflicting provider data, holiday hold, overcapacity evidence, or turnover risk.
- Draft internal staff tasks, daily-brief capacity risks, and manager-review summaries.
- Draft waitlist or customer-message recommendations without sending.

Staff review required:

- Applying a room assignment draft when inventory is limited, segmented, or close to full.
- Releasing tentative holds that are not manager-only.
- Moving a request to waitlist when staff needs to explain options to a customer.
- Confirming accommodation alternatives or split stays with a customer.
- Treating stale-but-recent provider data as sufficient for non-final internal planning.

Manager approval required:

- Any overbooking or decision that would exceed configured capacity.
- Releasing manager/holiday/blackout holds.
- Overriding minimum-stay, holiday, waitlist-only, or closure policies.
- Denying a request for capacity reasons where a customer/member-facing explanation will be sent.
- Resolving provider conflicts that could affect live reservations.
- Any payment/deposit/cancellation exception coupled to a capacity decision.

Never fully automate:

- Confirming, cancelling, modifying, or denying a live customer reservation.
- Promising availability or room type to a customer.
- Charging/refunding/waiving deposits because capacity is scarce.
- Sending sensitive explanations about denial, safety, incidents, or policy exceptions.
- Hiding capacity, safety, or care-risk evidence from staff/customer review artifacts.

### Review gates

Recommended review gates:

- `policy::ReviewGate::CapacityException`
- `policy::ReviewGate::ManagerHoldRelease`
- `policy::ReviewGate::HolidayPeakOverride`
- `policy::ReviewGate::WaitlistDecision`
- `policy::ReviewGate::ProviderDataConflict`
- `policy::ReviewGate::CustomerFacingAvailabilityMessage`
- `policy::ReviewGate::DepositOrCancellationException`

If these exact enum variants do not exist, later code should add semantically equivalent variants rather than reusing vague flags.

### Audit trail

Every decision that could affect a reservation or customer expectation must record:

- Actor: human staff/manager, agent, scheduled job, provider webhook, or import.
- Location and policy/contract version.
- Request identity or reservation identity.
- Date range and evaluated accommodation segments.
- Evidence refs and source timestamps.
- Decision variant and typed reasons.
- Review gate and approval state.
- Resulting staff tasks/recommended actions/tool drafts.
- Whether the output was internal-only, staff-facing, manager-facing, or customer/member-facing.

### Customer/member-facing boundaries

- Capacity outputs default to internal/staff-facing.
- Customer-facing availability text must be generated from an approved decision and reviewed if the outcome is denial, waitlist, policy exception, limited inventory, or accommodation substitution.
- Public website or chatbot flows may collect preferences and say a request is being reviewed; they must not promise exact room availability without deterministic policy and approval-cleared provider execution.

## 6. Test contracts

Future implementation should add semantic tests with names like these. They are contracts, not required to exist in this docs-only card.

### Construction and invariants

- `boarding_capacity_room_count_rejects_zero_inventory`
  - `capacity::RoomCount::try_new(0)` fails; closures/holds do not masquerade as zero inventory.
- `boarding_capacity_inventory_requires_at_least_one_segment`
  - Empty capacity inventory cannot construct for an active Boarding contract.
- `boarding_capacity_inventory_rejects_duplicate_accommodation_segments`
  - Classic, luxury, and cat segments are unique typed keys.
- `boarding_capacity_available_count_reports_overcapacity_instead_of_saturating_silently`
  - Booked greater than capacity produces typed overcapacity evidence.
- `boarding_capacity_occupancy_basis_points_remains_bounded_for_normal_capacity`
  - Normal occupancy is 0..=10_000; overcapacity uses `OverCapacityBy`.

### Accommodation and species safety

- `boarding_capacity_policy_never_uses_cat_condo_for_dog_request`
  - Dog request cannot consume cat segment capacity.
- `boarding_capacity_policy_never_uses_dog_suite_for_cat_request`
  - Cat request cannot consume dog segment capacity.
- `boarding_capacity_policy_respects_customer_accommodation_flexibility`
  - Exact requests are not silently downgraded/upgraded; flexible requests can evaluate eligible alternatives.
- `boarding_multi_pet_request_routes_split_accommodation_to_staff_review`
  - Multi-pet capacity split creates review rather than customer-facing auto-confirmation.

### Date range and per-night evaluation

- `boarding_capacity_decision_evaluates_every_night_in_stay_range`
  - Availability on arrival day alone is insufficient.
- `boarding_capacity_decision_waitlists_when_any_required_night_is_full`
  - Full segment on any night prevents direct availability.
- `boarding_capacity_snapshot_uses_location_local_boarding_nights`
  - Local date basis is explicit and stable across timezone boundaries.
- `boarding_capacity_policy_rejects_invalid_stay_range_before_inventory_lookup`
  - Bad date range fails before provider queries/drafts.

### Holds, closures, freshness, and provider conflicts

- `boarding_capacity_decision_routes_manager_hold_to_manager_review`
  - Manager holds are not automatically released.
- `boarding_capacity_decision_treats_maintenance_closure_as_unavailable_capacity`
  - Closed rooms reduce availability without changing configured inventory.
- `boarding_capacity_decision_blocks_live_confirmation_when_snapshot_is_stale`
  - Stale evidence may produce internal risk tasks, not live confirmation.
- `boarding_capacity_provider_conflict_creates_reconciliation_task`
  - Conflicting provider/local counts produce typed review and staff task.
- `boarding_capacity_overbooking_requires_manager_review`
  - Exceeding capacity cannot be automated.

### Season, holiday, and waitlist behavior

- `boarding_capacity_policy_applies_holiday_hold_before_available_decision`
  - Holiday/peak hold can route to waitlist/manager review even when count remains.
- `boarding_capacity_policy_waitlists_when_segment_is_waitlist_only`
  - Waitlist-only posture is not treated as limited/open capacity.
- `boarding_capacity_policy_denies_blackout_period_without_manager_override`
  - Blackout dates produce denial or manager review, not availability.
- `boarding_capacity_decision_preserves_waitlist_reason`
  - Waitlist output carries typed reason and evidence.

### Workflow, approval, and audit

- `boarding_capacity_alert_creates_internal_staff_task_not_customer_message`
  - Agent capacity alert remains internal.
- `boarding_capacity_assignment_draft_requires_staff_or_manager_approval_before_tool_execution`
  - Assignment drafts are not live room assignments.
- `boarding_capacity_decision_customer_facing_message_requires_review_gate`
  - Availability/denial/waitlist customer message cannot be sent directly from agent output.
- `boarding_capacity_audit_records_policy_version_evidence_refs_actor_and_review_gate`
  - Audit trail preserves why the decision happened.
- `boarding_capacity_decision_maps_to_resort_daily_brief_without_losing_segment_context`
  - Manager brief includes accommodation segment, not a flattened occupancy number only.

### Compatibility and serialization

- `boarding_contract_standard_petsuites_capacity_plan_remains_storage_roundtrippable`
  - Existing `Contract::standard_petsuites()` storage compatibility is preserved during refactor.
- `boarding_capacity_segmented_inventory_serializes_without_flattening_accommodation_kind_to_stringly_labels`
  - Storage conversion preserves typed dog/cat/classic/luxury meaning.
- `boarding_capacity_refactor_keeps_room_inventory_as_compatibility_projection_until_migration_completes`
  - Old contract readers have a clear migration path.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add or refine `operations::boarding::capacity`, `accommodation`, `room`, `season`, and workflow-planning child modules.
  - Keep `operations::boarding::Contract` as root and preserve `standard_petsuites()`.
- `domain/src/entities.rs`
  - Likely consume existing `LocationId`, `PetId`, `CustomerId`, `Reservation`, `Species`; avoid duplicating them in Boarding.
- `domain/src/workflow.rs`
  - Add or refine recommended-action/review/audit event variants if existing workflow vocabulary is too generic.
- `domain/src/policy.rs`
  - Add review gates/automation levels for capacity exceptions if missing.
- `domain/src/tools.rs`
  - Add/adjust availability, room-assignment, waitlist, or reservation-update draft boundaries if provider execution is modeled.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Preserve current contract tests; add capacity segmentation and compatibility tests.
- `domain/tests/domain_quality_patterns.rs`
  - Add semantic-pattern assertions if it checks for raw strings/helper-shaped anti-patterns.
- Future storage adapter files under `storage::operations` if present in this workspace/codebase.
  - Contract serialization/migration is the highest-risk integration point.

### Migration/refactor risks

- Current `operations::boarding::RoomInventory` is a single positive count. Replacing it abruptly with segmented inventory could break contract builders and storage codecs. Prefer an additive `capacity::Inventory` plus compatibility projection first.
- Current `RoomAvailability` may be too coarse. Do not overload variants with hidden reasons; introduce typed reasons under `capacity::Availability`.
- Existing root `operations::BoardingAccommodation` is flat. Moving directly to nested dog/cat accommodation types can break root `ServiceOffering::Boarding` serialization. Provide conversion and roundtrip tests.
- Avoid a generic `capacity` helper shared across boarding/daycare/grooming too early. Boarding capacity is overnight-room/date-range/segment-specific; daycare capacity and grooming utilization have different semantics.
- Do not let provider DTOs or raw room labels leak into domain decisions. Convert at the repository/storage boundary.
- Do not encode overbooking by allowing occupancy > 10_000 basis points in a normal scalar. Use explicit overcapacity evidence and manager review.
- Do not conflate configured room inventory with currently usable capacity. Holds/closures/turnover reduce availability, not the configured inventory count.
- Do not put customer messaging behavior on `capacity::Decision`; decision can expose reasons, but reviewed message drafting belongs in workflow/agent/customer-message boundaries.

### Dependencies on other implications

- Stay/date-range and minimum-stay policy implication: capacity decisions depend on valid `stay::DateRange`, derived nights, and peak minimum-stay overlays.
- Deposit/cancellation implication: capacity may determine whether confirmation can proceed, but deposit/cancellation policies own money and exception outcomes.
- Care/playtime/accommodation implication: care and species constraints can restrict accommodation eligibility.
- Staff task/handoff implication: limited capacity, room readiness, and hold release must create operable staff tasks.
- Pawgress/customer communication implication: capacity denial/waitlist/availability messages are customer-facing and require review boundaries.
- Agent approval implication: capacity alerts and website reservation assistants need typed automation levels and review gates.

### Recommended implementation sequence

1. Add `operations::boarding::accommodation::{Kind, DogSuiteKind, CatAccommodationKind, Flexibility}` plus conversions from existing `operations::BoardingAccommodation`.
2. Add `operations::boarding::capacity::{RoomCount, BookedCount, AvailableCount, Inventory, Segment, Snapshot, Availability, Decision}` with construction tests.
3. Keep `Contract.capacity: CapacityPlan` initially, but add `capacity_policy`/segmented inventory hooks or conversion methods so storage migration can be staged.
4. Implement `capacity::Policy::evaluate` against in-memory snapshots only; no provider writes.
5. Add repository traits/ports for snapshots, audit, hold drafts, and assignment drafts.
6. Add workflow planning and approval-policy tests that prove agents create internal recommendations, not live booking/customer actions.
7. Only then wire provider/storage adapters and serialized Rust records with explicit migration tests.
