# Boarding implication 02: Room/suite availability

Purpose: model how PetSuites Boarding should decide and communicate room/suite availability without turning an AI recommendation into a live booking, live room assignment, customer promise, or payment action. This is a domain-contract artifact for later Rust work.

Source context:

- `docs/domain/petsuites/boarding/service-domain-map.md`
- Current `domain/src/operations.rs` Boarding surface: `operations::boarding::Contract`, `CapacityPlan`, `RoomInventory`, `RoomAvailability`, `ServiceWindow`, `MinimumStay`, `CancellationPolicy`, `DepositRule`, `HousekeepingCadence`, `HandoffRequirement`, and broad `operations::BoardingAccommodation` / `ServiceOffering::Boarding`.
- Current tests in `domain/tests/petsuites_core_service_contracts.rs` and `domain/tests/domain_quality_patterns.rs`, especially semantic scalar construction, typed availability decisions, workflow recommendations, and storage roundtrip pressure.

Assumption: exact room counts, suite names, local holiday holds, and provider-room identifiers are location data. The domain core should encode typed capacity semantics and safe review gates; it should not hard-code a PetSuites-wide inventory or make availability promises from stale/partial data.

## 1. Operational story

### Trigger

Room/suite availability evaluation is triggered when any of these happens:

1. A customer, staff member, website assistant, or reservation workflow asks whether a boarding stay can be accepted for a location, date range, pet set, and accommodation preference.
2. A reservation is created, changed, cancelled, checked in, checked out, or waitlisted and capacity projections must be recalculated.
3. A manager/day brief/capacity-alert workflow needs a snapshot of upcoming occupancy, limited segments, holds, and waitlist pressure.
4. A local policy change creates or removes peak/holiday minimum stays, manager holds, blackout periods, or room-segment closures.

### Actors

- Customer/member: asks for a stay or change, but does not directly modify room inventory in the domain model.
- Front desk / reservation staff: reviews availability evidence, offers options, applies approved holds, and communicates only approved outcomes.
- Kennel / lodging lead: owns operational fit of room/suite assignments, turnover readiness, cleaning status, pet-to-room suitability, and special-handling flags.
- Manager: approves capacity overrides, overbooking exceptions, holiday holds, waitlist promotions, suite upgrades in limited inventory, and customer-facing denials/explanations when sensitive.
- AI/workflow agent: reads snapshots, drafts internal recommendations/tasks/messages, detects risks, and explains deterministic policy outcomes. It cannot confirm a booking, place a live room hold, assign a room, or send customer availability promises without typed approval.
- Provider/storage adapter: exposes reservation projections, room inventory records, holds, closures, and external room IDs through semantic repository ports.

### Inputs

- `entities::LocationId` and the location's Boarding contract/policy version.
- `boarding::stay::DateRange` with check-in/check-out local dates/times and derived positive `stay::Nights`.
- One or more pets: `entities::PetId`, `entities::Species`, care/temperament projections, and optional same-suite compatibility facts for multi-pet stays.
- Requested accommodation preference: specific `accommodation::Kind`, an acceptable set, or a flexibility policy such as classic-or-luxury, dog-suite-any, or cat-only.
- Current capacity evidence: segment inventory, booked counts, existing holds, waitlist counts, room closures, turnover/cleaning state, and timestamp/source freshness.
- Season and policy context: peak/holiday period, blackout/local event, minimum stay, deposit timing, cancellation risk, manager hold rules.
- Reservation context when changing an existing stay: reservation ID/status, current accommodation/hold, deposit state, check-in state, and prior manager approvals.

### Decisions

The truthful owner is `operations::boarding::capacity::Policy`, composed by a broader `boarding::policy::Evaluator` when stay, care, deposit, and workflow outputs must be produced together.

The capacity policy decides:

- Which room/suite segments are relevant for the requested pets and accommodation preference.
- Whether the date range satisfies segment-specific inventory and local holds for every night.
- Whether availability is open, limited, waitlist-only, closed, or manager-held.
- Whether an internal hold can be drafted, must be reviewed, or is denied.
- Whether an accommodation mismatch exists: cat to dog suite, dog to cat condo, multi-pet incompatibility, or special-care requirement that narrows eligible rooms.
- Whether stale/incomplete provider data makes the result review-only rather than available.
- Whether a status recommendation should be `ReadyForStaffApproval`, `Waitlisted`, `ManagerReviewRequired`, or `Denied`.

### Outputs

- `boarding::capacity::Snapshot`: typed evidence for location/date range/segment, including inventory, booked counts, holds, closures, occupancy, freshness, and source.
- `boarding::capacity::Decision`: deterministic result with typed reason and evidence references.
- `boarding::accommodation::AssignmentDraft`: internal proposed assignment or hold, never a live room assignment until approved/executed through a tool boundary.
- `boarding::workflow::AvailabilityPlan`: staff tasks, status-update drafts, customer-message drafts, and manager-review gates created from the decision.
- `workflow::RecommendedAction` values for reservation status or internal task creation.
- `operations::OperationsRisk::CapacityConstraint` / daily-brief entries when availability is limited, stale, held, closed, or waitlist-heavy.
- Audit events tying actor, request, policy version, snapshot source, decision, approval state, and any outbound draft together.

### Success state

A successful availability evaluation produces a typed decision from fresh enough evidence, preserves segment semantics, and leaves the system in one of these safe states:

- `AvailableForStaffApproval`: capacity exists and an internal hold/assignment draft can be reviewed.
- `WaitlistRecommended`: no safe open capacity exists, but a waitlist route is valid.
- `ManagerReviewRequired`: capacity might exist but a local hold, override, limited inventory, stale evidence, special care, multi-pet assignment, or customer-impacting exception requires approval.
- `DeniedByPolicy`: deterministic policy says no safe availability path exists.

No success state sends a customer-facing promise by itself. Customer-facing confirmation requires separate approval and execution boundaries.

### Failure and exception states

- `UnknownInventory`: no inventory record exists for the location/segment/date range.
- `StaleSnapshot`: provider data is older than the contract's freshness threshold.
- `AccommodationSpeciesMismatch`: requested room/suite does not match pet species.
- `SegmentClosed`: every relevant room segment is closed for the requested dates.
- `RoomOutOfService`: only room-level capacity exists but eligible rooms are closed, dirty, under repair, or not turnover-ready.
- `HolidayHoldRequiresManager`: peak/holiday rules reserve remaining rooms for manager review.
- `MinimumStayViolation`: requested nights do not satisfy standard or seasonal minimum stay.
- `OverCapacity`: booked plus held counts reach or exceed inventory for at least one night.
- `MultiPetCompatibilityUnknown`: same-room/suite assumptions are missing or unsafe.
- `SpecialCareNarrowsInventory`: medication, mobility, anxiety, isolation, or behavior facts require staff/manager review before assigning an ordinary room.
- `ProviderWriteUnavailable`: read succeeds but hold/assignment draft cannot be executed; create internal task and do not promise availability.
- `ConflictingEvidence`: provider reservations, staff holds, and internal snapshots disagree.

## 2. Domain types to add or refine

Prefer semantic modules under `operations::boarding`, keeping the existing `boarding::Contract` as the service-line root and refactoring in place rather than creating parallel root types.

### Accommodation

- `operations::boarding::accommodation::Kind`
  - `Dog(DogSuiteKind)` or `Cat(CatAccommodationKind)`.
  - Invariant: species compatibility is encoded by variant, not string labels.
- `operations::boarding::accommodation::DogSuiteKind`
  - `ClassicSuite`, `LuxurySuite`.
- `operations::boarding::accommodation::CatAccommodationKind`
  - `CatCondo`; extensible for future cat suite products.
- `operations::boarding::accommodation::Preference`
  - `Specific(Kind)`, `AnyDogSuite`, `AnyOf(Vec<Kind>)`, `ExistingAssignment(accommodation::AssignmentId)`.
  - Invariant: empty `AnyOf` is invalid; cat flexibility cannot include dog suites for cats unless a future cross-species policy explicitly exists.
- `operations::boarding::accommodation::RoomId`
  - Non-empty provider/location room identity; never a raw string at domain call sites.
- `operations::boarding::accommodation::AssignmentDraft`
  - Proposed room/suite/segment assignment for a reservation/pet/date range.
  - Invariants: covers the whole stay date range or explicitly records split-stay review; carries approval state; cannot claim live assignment.

### Capacity and inventory

- `operations::boarding::capacity::Inventory`
  - Segmented inventory by `accommodation::Kind` and effective date or version.
  - Invariant: every configured segment has positive `RoomCount`.
- `operations::boarding::capacity::RoomCount`
  - Positive scalar; refines current `boarding::RoomInventory` for segment-specific capacity.
- `operations::boarding::capacity::BookedCount`
  - Non-negative scalar for reservations consuming capacity.
- `operations::boarding::capacity::HeldCount`
  - Non-negative scalar for staff/manager/provider holds; separate from booked count.
- `operations::boarding::capacity::WaitlistCount`
  - Non-negative scalar; does not consume room inventory.
- `operations::boarding::capacity::OccupancyBasisPoints`
  - 0..=10_000 for normal occupancy. If overbooking must be represented, use `OverCapacityBy(RoomCount)` instead of allowing silent >100% saturation.
- `operations::boarding::capacity::SnapshotId`
  - Non-empty source/evidence identifier, separate from generic daily-brief `operations::SnapshotId` if provider snapshots need stronger provenance.
- `operations::boarding::capacity::SnapshotFreshness`
  - `Fresh`, `Stale`, `Unknown`; policy decides whether stale evidence permits draft-only recommendations.
- `operations::boarding::capacity::SegmentSnapshot`
  - Segment, inventory, booked, held, closed/out-of-service counts, availability state, snapshot ID, observed-at timestamp.
- `operations::boarding::capacity::Snapshot`
  - Location, date range, all segment snapshots, source, policy version, and freshness.
- `operations::boarding::capacity::Availability`
  - Refines current `RoomAvailability`: `Open`, `Limited`, `WaitlistOnly`, `Closed`, `ManagerHold`, `Unknown`.
- `operations::boarding::capacity::Decision`
  - `Available { evidence, hold: HoldDraft, assignment: AssignmentDraft }`
  - `Limited { evidence, review_gate }`
  - `Waitlist { evidence, reason }`
  - `Deny { evidence, reason }`
  - `ManagerReview { evidence, reason }`
  - `Unknown { evidence, reason }`
  - Invariant: no boolean `available` without reason/evidence.
- `operations::boarding::capacity::DenialReason`
  - `SegmentClosed`, `MinimumStayViolation`, `SpeciesAccommodationMismatch`, `OverCapacity`, `BlackoutPeriod`, `PolicyUnavailable`, `UnsupportedLocation`.
- `operations::boarding::capacity::WaitlistReason`
  - `NoSegmentCapacity`, `HolidayDemand`, `PendingCancellation`, `ManagerHold`, `RequestedUpgradeUnavailable`.
- `operations::boarding::capacity::ReviewReason`
  - `LimitedInventory`, `StaleSnapshot`, `HolidayHold`, `ProviderConflict`, `MultiPetFit`, `SpecialCareRoomFit`, `OverrideRequested`, `RoomClosureConflict`.

### Holds and room lifecycle

- `operations::boarding::capacity::HoldDraft`
  - Internal proposed hold with expiration, segment, date range, pet count, source actor, and approval requirement.
  - Invariant: draft cannot be confused with live provider hold.
- `operations::boarding::capacity::HoldState`
  - `Draft`, `StaffApproved`, `ManagerApproved`, `Applied`, `Expired`, `Released`, `Rejected`.
- `operations::boarding::room::Status`
  - `Reservable`, `Occupied`, `TurnoverRequired`, `OutOfService`, `ManagerHold`, `Unknown`.
- `operations::boarding::room::TurnoverState`
  - `CleanReady`, `CleaningScheduled`, `CleaningBlocked`, `InspectionRequired`.
- `operations::boarding::room::Compatibility`
  - Deterministic result for pet/accommodation/room fit, with explicit review reasons.

### Stay request and policy composition

- `operations::boarding::StayRequest`
  - Required: location, customer, pets, date range, accommodation preference, requested add-ons, request source.
  - Invariant: date range positive; at least one pet; requested accommodation compatible enough to evaluate.
- `operations::boarding::StayPlan`
  - Capacity decision plus care/deposit/stay tasks required before staff can confirm.
- `operations::boarding::stay::DateRange`
  - Check-out follows check-in, derived positive `stay::Nights`, location timezone explicit when local date boundaries matter.
- `operations::boarding::season::Period` and `season::Policy`
  - Named peak/holiday/blackout/local event rules that can force minimum stays or manager holds.

### Errors

- `operations::boarding::capacity::Error`
  - Module-local semantic error, not `String`/`anyhow` in core.
  - Variants should carry typed context: `InventoryMissing { location_id, segment }`, `SnapshotStale { snapshot_id }`, `AccommodationMismatch { pet_id, requested }`, `ProviderConflict { snapshot_id }`.
- `operations::boarding::Result<T>` or child-module result aliases as the surface grows.

## 3. Relationship map

### Entities and value objects

- `entities::LocationId` selects a Boarding contract, inventory set, room closures, local policies, and timezone.
- `entities::CustomerId` anchors request ownership and customer-message drafts, but Boarding should not copy customer contact details into capacity decisions.
- `entities::PetId`, `entities::Species`, `TemperamentProfile`, and `CareProfile` shape accommodation compatibility and review gates.
- `entities::ReservationId` links capacity decisions, holds, status-update recommendations, and audit events to the cross-service reservation lifecycle.
- `boarding::stay::DateRange`, `stay::Nights`, `accommodation::Kind`, `capacity::RoomCount`, `BookedCount`, `HeldCount`, and `OccupancyBasisPoints` are the semantic values that prevent raw primitive capacity logic.

### Policies

- `boarding::capacity::Policy` owns capacity/hold/availability decisions.
- `boarding::season::Policy` supplies peak/holiday/blackout context and local manager holds.
- `boarding::stay::Policy` owns minimum-stay and check-in/check-out date-window validity.
- `boarding::care::Policy` can narrow eligible room/suite choices for medical, behavior, anxiety, mobility, feeding, or medication concerns.
- `boarding::agent::ApprovalPolicy` maps capacity outputs into automation levels and review gates.
- `policy::AutomationLevel`, `policy::ReviewGate`, and `workflow::PolicyContext` provide the cross-domain approval envelope.

### Repositories and stores

- `boarding::Repository` loads the location-scoped Boarding contract and policy version.
- `boarding::capacity::Repository` loads segmented capacity snapshots and persists internal hold drafts or approved hold projections.
- `boarding::room::Repository` loads room status, turnover state, closures, and room-level assignment projections.
- `boarding::reservation::Repository` queries reservation projections by location/date/service/accommodation segment.
- `storage::operations::CoreServiceContractsRecord` persists `boarding::Contract` and must keep serializing/deserializing during refactors.
- Provider adapters convert raw portal inventory/room JSON into semantic `capacity::Snapshot` values at the boundary.

### Workflow events and staff tasks

- `workflow::WorkflowEventType::BookingTriageNeeded` and future `BoardingCapacityReviewNeeded` trigger policy evaluation.
- `workflow::RecommendedAction::UpdateStatus` carries reservation status suggestions such as `Waitlisted` with `TransitionIntent::ApplyCapacityDecision`.
- `workflow::RecommendedAction::InternalTask` and `operations::StaffTask` carry review/hold/turnover tasks.
- Boarding-owned task kinds should map to generic staff tasks without losing source semantics:
  - `boarding::task::Kind::ReviewLimitedAvailability`
  - `boarding::task::Kind::ApproveHolidayHoldRelease`
  - `boarding::task::Kind::VerifyRoomTurnoverReady`
  - `boarding::task::Kind::ResolveProviderCapacityConflict`
  - `boarding::task::Kind::PromoteWaitlistCandidate`

### Agent specs and tools

- `agents::WorkflowAgent` may run a capacity-alert or booking-triage workflow using read-only availability/portal tools.
- `tools::AvailabilityRequest` / `AvailabilityResult` are broad integration envelopes; Boarding should wrap/translate them through `boarding::capacity` before core policy depends on them.
- `tools::ReservationUpdateDraft` is the correct boundary for status changes; agents should not mutate `entities::Reservation` directly.
- Future tool specs should separate:
  - `boarding-capacity-read`: read-only capacity snapshot.
  - `boarding-hold-draft`: creates an internal draft only.
  - `boarding-hold-apply`: requires staff/manager approval token.
  - `boarding-room-assignment-draft`: draft assignment only.
  - `boarding-customer-message-draft`: draft with review gate.

## 4. Interaction contract

Rust-like pseudo-signatures below describe ownership and behavior. They are not implementation mandates for one code card.

```rust
impl boarding::StayRequest {
    pub fn builder() -> stay_request::Builder<MissingLocation, MissingPets, MissingDateRange>;

    pub fn requested_accommodation(&self) -> &accommodation::Preference;
    pub fn date_range(&self) -> &stay::DateRange;
    pub fn pets(&self) -> &[entities::PetId];
}
```

`StayRequest` owns construction validity for a boarding request. It does not own capacity lookup.

```rust
impl stay::DateRange {
    pub fn new(
        check_in: stay::LocalCheckIn,
        check_out: stay::LocalCheckOut,
        timezone: location::Timezone,
    ) -> stay::Result<Self>;

    pub fn nights(&self) -> stay::Nights;
    pub fn overlaps(&self, period: &season::Period) -> bool;
}
```

`DateRange` owns chronological validity and derived nights.

```rust
pub trait boarding::Repository {
    fn contract_for(
        &self,
        location: entities::LocationId,
    ) -> boarding::Result<boarding::Contract>;
}

pub trait capacity::Repository {
    fn snapshot_for(
        &self,
        request: &capacity::SnapshotRequest,
    ) -> capacity::Result<capacity::Snapshot>;

    fn draft_hold(
        &self,
        draft: capacity::HoldDraft,
    ) -> capacity::Result<capacity::HoldDraftId>;
}

pub trait room::Repository {
    fn room_statuses(
        &self,
        location: entities::LocationId,
        range: stay::DateRange,
        segment: accommodation::Kind,
    ) -> room::Result<Vec<room::StatusSnapshot>>;
}

pub trait reservation::Repository {
    fn boarding_reservations(
        &self,
        query: reservation::BoardingCapacityQuery,
    ) -> reservation::Result<Vec<reservation::CapacityProjection>>;
}
```

Repositories own data access. They do not decide policy. `draft_hold` produces an internal draft/ID only; applying a provider hold belongs to an approved tool adapter.

```rust
impl capacity::Inventory {
    pub fn room_count(&self, kind: accommodation::Kind) -> Option<capacity::RoomCount>;
    pub fn configured_segments(&self) -> impl Iterator<Item = accommodation::Kind>;
}

impl capacity::SegmentSnapshot {
    pub fn availability(&self, policy: &capacity::Policy) -> capacity::Availability;
    pub fn occupancy(&self) -> capacity::OccupancyBasisPoints;
    pub fn remaining_rooms(&self) -> capacity::RemainingRooms;
}
```

Inventory and snapshots own arithmetic and scalar invariants. They should not return untyped tuples like `(booked, total, available)`.

```rust
impl capacity::Policy {
    pub fn decide(
        &self,
        request: &boarding::StayRequest,
        contract: &boarding::Contract,
        snapshot: &capacity::Snapshot,
        season: &season::Decision,
        care_fit: &room::Compatibility,
    ) -> capacity::Decision;
}
```

`capacity::Policy` owns deterministic availability. It returns `Decision`, not side effects.

```rust
impl room::CompatibilityPolicy {
    pub fn evaluate(
        &self,
        pets: &[entities::Pet],
        care_profiles: &[entities::CareProfile],
        temperament_profiles: &[entities::TemperamentProfile],
        requested: &accommodation::Preference,
        room_statuses: &[room::StatusSnapshot],
    ) -> room::Compatibility;
}
```

Room compatibility owns species, room-status, special-care, and multi-pet fit. It should not be hidden inside a generic helper or availability boolean.

```rust
impl boarding::policy::Evaluator {
    pub fn evaluate_stay_request(
        &self,
        request: boarding::StayRequest,
    ) -> boarding::Result<boarding::StayPlan>;
}
```

The evaluator composes repositories and policies. It can create an internal plan but should not confirm a booking.

```rust
impl boarding::workflow::Planner {
    pub fn plan_from_capacity_decision(
        &self,
        request: &boarding::StayRequest,
        decision: &capacity::Decision,
    ) -> workflow::Result<Vec<workflow::RecommendedAction>>;

    pub fn staff_tasks_for(
        &self,
        decision: &capacity::Decision,
    ) -> Vec<operations::StaffTask>;
}
```

The workflow planner owns staff tasks and recommended actions generated from decisions.

```rust
impl boarding::agent::ApprovalPolicy {
    pub fn classify_capacity_action(
        &self,
        action: &boarding::agent::CapacityAction,
        decision: &capacity::Decision,
    ) -> workflow::PolicyContext;
}
```

The approval policy owns automation boundaries. Tool adapters should require this context before executing writes.

## 5. Review and approval contract

### Automation level

Safe/internal automation:

- Read capacity snapshots and reservation projections.
- Compute deterministic `capacity::Decision` from fresh evidence.
- Draft internal staff tasks for limited availability, waitlist review, room turnover, provider conflicts, and manager holds.
- Draft capacity-alert/daily-brief summaries.
- Draft customer-message text that is clearly marked draft-only.

Staff approval required:

- Applying an ordinary room/suite hold when evidence is fresh, inventory is open, no special-care review is pending, and no local hold exists.
- Offering a customer a specific room/suite option or upgrade.
- Promoting a waitlist candidate when cancellation/open capacity appears.
- Marking room turnover/cleaning evidence as ready when evidence is free-text/media/manual.
- Sending customer-facing availability messages that do not involve denial, payment, medical, behavior, incident, or policy exception content.

Manager approval required:

- Overriding capacity, overbooking, releasing holiday holds, bypassing blackout/closed states, or changing waitlist priority.
- Confirming availability from stale/conflicting provider data.
- Approving special-care room fit, multi-pet same-suite exceptions, or behavior/medical accommodations that affect safety.
- Customer-facing denials, policy explanations, sensitive incident/medical/behavior/payment availability messages.
- Waiving deposit/minimum-stay/cancellation rules to make an availability path work.

Never fully automate:

- Confirming, cancelling, rejecting, or modifying a live reservation.
- Charging, refunding, waiving, or forfeiting deposits.
- Applying a live provider hold/room assignment without approved workflow context.
- Sending member-facing promises or denials without review.
- Hiding negative safety, care, or capacity facts from the audit trail.

### Review gates

Suggested review-gate mapping:

- `policy::ReviewGate::StaffApproval`: ordinary hold/assignment/customer-message drafts.
- `policy::ReviewGate::ManagerApproval`: capacity override, holiday hold release, stale snapshot, provider conflict, denial explanations, waitlist priority, special-care fit.
- `policy::ReviewGate::MedicalOrCareReview` if introduced: medication/mobility/special-room fit before assignment.
- `policy::ReviewGate::PaymentOrDepositReview` if introduced: deposit/minimum-stay exception used to create availability.

Absence of a review gate is not permission. The domain should carry an explicit `workflow::PolicyContext` for any action that crosses a side-effect boundary.

### Audit trail

Every availability evaluation should record or be able to derive:

- Actor: customer, staff, manager, system, or agent.
- Location, date range, requested accommodation preference, and pet/reservation references.
- Contract version, capacity policy version, season policy version, and snapshot IDs/timestamps.
- Capacity decision variant and typed reason.
- Review gates required and who cleared them.
- Drafts created: hold draft, assignment draft, status-update draft, staff task, or customer-message draft.
- Tool execution result, if an approved adapter later applies a hold/assignment/status update.

### Customer/member-facing boundary

A `capacity::Decision::Available` means the domain found capacity evidence. It does not mean the customer has a confirmed reservation. Customer-facing text should use approved language such as "staff can review this option" until an approved reservation/hold execution succeeds.

## 6. Test contracts

Future implementation cards should add semantic tests like these before or alongside code:

- `boarding_capacity_inventory_segments_classic_luxury_and_cat_condo_counts`
  - Inventory preserves separate counts per accommodation kind and does not collapse all rooms into one raw total.
- `boarding_capacity_inventory_rejects_zero_configured_segment_count`
  - `capacity::RoomCount::try_new(0)` fails for configured inventory.
- `boarding_capacity_snapshot_allows_zero_booked_and_zero_holds`
  - Booked/held/waitlist counts can be zero while configured inventory remains positive.
- `boarding_capacity_policy_returns_available_with_hold_draft_for_fresh_open_segment`
  - Fresh open capacity returns `Decision::Available` with evidence and a draft hold, not a live booking.
- `boarding_capacity_policy_routes_limited_inventory_to_staff_or_manager_review`
  - Limited inventory carries a review gate instead of direct assignment.
- `boarding_capacity_policy_waitlists_when_requested_segment_is_full_for_any_stay_night`
  - A full night inside the date range prevents an available decision.
- `boarding_capacity_policy_denies_when_all_eligible_segments_are_closed`
  - Closed/out-of-service segments produce typed denial.
- `boarding_capacity_policy_marks_stale_snapshot_unknown_or_manager_review`
  - Stale evidence never produces customer-promise-ready availability.
- `boarding_capacity_policy_preserves_holiday_hold_as_manager_review_reason`
  - Peak/holiday holds are explicit reasons, not generic unavailability.
- `boarding_room_assignment_cannot_assign_cat_to_dog_suite_or_dog_to_cat_condo`
  - Species/accommodation mismatch is impossible at construction or returns a typed denial.
- `boarding_room_compatibility_routes_special_care_room_fit_to_review`
  - Medication/mobility/anxiety/isolation flags can narrow eligibility and create review tasks.
- `boarding_multi_pet_same_suite_requires_explicit_compatibility_evidence`
  - Multi-pet assignments do not assume same-suite fit without typed evidence.
- `boarding_minimum_stay_violation_blocks_capacity_hold_draft`
  - Stay policy failure prevents hold draft creation even if rooms are open.
- `boarding_availability_decision_maps_waitlist_to_reservation_status_draft_only`
  - Waitlist result creates `workflow::RecommendedAction::UpdateStatus`, not direct reservation mutation.
- `boarding_availability_planner_creates_staff_task_for_provider_capacity_conflict`
  - Conflicting source evidence produces an internal task with the capacity snapshot source.
- `boarding_capacity_alert_daily_brief_preserves_boarding_segment_context`
  - Daily brief risk retains Boarding/accommodation segment meaning.
- `boarding_agent_can_read_capacity_and_draft_tasks_but_cannot_apply_live_hold`
  - Approval policy forbids live hold application without approval context.
- `boarding_customer_availability_message_remains_draft_until_staff_approved`
  - Customer message draft cannot be sent from draft state.
- `boarding_capacity_override_requires_manager_approval_audit_event`
  - Override/overbooking/holiday hold release carries manager gate and audit metadata.
- `boarding_capacity_snapshot_roundtrips_through_storage_without_raw_provider_json_leaking_into_domain`
  - Storage adapters convert to semantic snapshots and preserve invariants on decode.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Keep `pub mod boarding` and `boarding::Contract` as the root service-line surface.
  - Add child modules incrementally: `boarding::accommodation`, `boarding::capacity`, `boarding::room`, `boarding::stay`, `boarding::season`, `boarding::workflow`, and `boarding::agent` only as behavior demands.
  - Re-export carefully so call sites can use `operations::boarding::Contract` and semantically nested leaves like `operations::boarding::capacity::Decision`.
- `domain/src/entities.rs`
  - Likely no ownership move, but capacity decisions reference `LocationId`, `PetId`, `ReservationId`, `Species`, `ReservationStatus`, and audit values.
- `domain/src/policy.rs`
  - May need more precise review gates for capacity, medical/care, payment/deposit, or keep using `ManagerApproval` / `StaffApproval` until pressure justifies new variants.
- `domain/src/workflow.rs`
  - May need a Boarding capacity workflow event or status-transition intent if current `BookingTriageNeeded` / `ApplyCapacityDecision` is too broad.
- `domain/src/tools.rs`
  - Availability and reservation update tools should remain boundary envelopes; add Boarding-specific wrappers only if provider execution needs stronger types.
- `storage/src/operations.rs`
  - `CoreServiceContractsRecord` currently stores `domain::operations::boarding::Contract`; serialization must continue to decode existing contracts or provide an explicit migration.
- `storage/tests/core_service_contract_storage.rs`
  - Add roundtrip/invalid-json tests for segmented capacity and semantic counts when the storage shape changes.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add contract-level tests for segmented room/suite availability.
- `domain/tests/domain_quality_patterns.rs`
  - Add tests that availability decisions are semantic enums with evidence, not boolean/string pairs.

### Migration and refactor risks

- Current `boarding::RoomInventory` is a single positive scalar. Segmented inventory should not break `Contract::standard_petsuites()` or existing tests before storage migration is ready. A safe path is to introduce `capacity::Inventory` alongside a compatibility constructor from the existing scalar, then migrate call sites.
- Current `boarding::RoomAvailability` lacks `ManagerHold` and `Unknown`. Adding variants can affect serialized enums. Prefer additive migration with storage tests and explicit backward compatibility.
- Broad `operations::BoardingAccommodation::{ClassicSuite, LuxurySuite, CatCondo}` is usable vocabulary, but behavior wants `accommodation::Kind::{Dog(DogSuiteKind), Cat(CatAccommodationKind)}`. Avoid maintaining two parallel canonical surfaces indefinitely; re-export or conversion should have one clear direction.
- `tools::AvailabilityResult` already models typed tool decisions. Do not let a tool-level `Available` bypass Boarding's richer segment/policy/staleness review.
- Room assignment and hold execution are side-effect boundaries. Keep draft domain values separate from provider-applied values.
- Adding typestate to the stay lifecycle is premature unless legal method progression cannot be enforced by ordinary enums/builders. Start with runtime semantic enums and tests.

### Dependencies on other Boarding implications

- Stay/minimum-stay/seasonality: capacity cannot decide availability for peak/holiday periods without `season::Policy` and `stay::Policy` inputs.
- Deposit/payment implication: availability may be open while confirmation is blocked by due-at-booking deposit; keep capacity and payment decisions separate but composable.
- Care/medication/behavior implication: room fit can be narrowed by care facts and may require staff/manager review before assignment.
- Check-in/check-out implication: room turnover and late checkout affect same-day availability and assignment readiness.
- Pawgress/customer messaging implication: availability explanations and denial messages are customer-facing and must follow draft/review/send boundaries.
- Staff handoff implication: manager holds, limited inventory, waitlists, and turnover blocks should feed shift handoff packets.

### Recommended implementation slice

1. Add `boarding::accommodation` and `boarding::capacity` types with semantic counts, segmented inventory, snapshots, decisions, and errors.
2. Add tests for scalar invariants, species/accommodation mismatch, stale snapshot behavior, limited/holiday review gates, and waitlist decisions.
3. Wire `boarding::Contract` to expose either current `CapacityPlan` or new segmented `capacity::Inventory` through a compatibility path.
4. Add storage roundtrip tests before changing serialized contract shapes.
5. Add workflow planner/approval-policy tests that prove agents can draft capacity tasks but cannot confirm reservations or apply live holds.
