# Boarding implication 03: Holiday demand spikes

Purpose: model the operational implication of holiday/peak demand spikes for PetSuites Boarding. This is a domain-contract artifact for later Rust implementation. It does not authorize live booking, customer messaging, payment, deposit waiver/refund, room assignment, or member-facing action.

Source context:

- `docs/domain/petsuites/boarding/service-domain-map.md`
- Existing `domain/src/operations.rs` Boarding contract surface: `operations::boarding::Contract`, `CapacityPlan`, `RoomInventory`, `RoomAvailability`, `MinimumStay`, `MinimumStayReason::HolidayPeak`, `CancellationPolicy`, `DepositRule`, `PaymentTiming`, `HousekeepingCadence`, `HandoffRequirement`, and `Upsell`.
- Existing generic surfaces: `entities::{LocationId, CustomerId, PetId, ReservationId, Reservation, ServiceKind}`, `operations::{StaffTask, StaffTaskKind, ResortDailyBrief, OperationsRisk, RevenueOpportunityKind}`, `tools::{AvailabilityRequest, AvailabilityResult, ReservationUpdateDraft}`, `workflow::{RecommendedAction, ReviewGate-like audit envelope}`, `agents::baseline_agent_specs()`.

Assumptions:

- Holiday periods, exact room counts, blackout dates, deposit amounts, cancellation deadlines, and minimum-night rules are location policy data. The domain model should encode them as typed, versioned policy inputs, not constants.
- A holiday demand spike may be forecasted before inventory is full, detected from waitlist/lead volume, or observed when same-day availability changes. The safest model treats all three as typed demand signals that produce internal recommendations until staff/manager approval clears an execution boundary.
- Capacity must remain segmented by accommodation kind. Dog classic suites, dog luxury suites, and cat condos share a stay lifecycle but not the same inventory pool, care assumptions, play eligibility, or room-assignment rules.
- Overbooking is not a normal domain state. If a location intentionally holds or exceeds capacity during a holiday, it must be represented as `ManagerReview`/`ManagerHold` evidence rather than as an automatically available room.

## 1. Operational story

### Trigger

A holiday demand spike begins when one or more location-scoped signals indicate that boarding demand during a named peak period is approaching or exceeding planned capacity:

- `season::Policy` marks requested dates as a holiday, school-break, local-event, blackout, or other `season::DemandClass` above normal.
- `capacity::Snapshot` reports high occupancy, rapidly shrinking availability, a full accommodation segment, or a manually configured manager hold.
- Lead/reservation inquiry volume rises for the same date range and accommodation segment.
- Existing reservations change in ways that increase operational load: multi-pet stays, late checkouts, medication-heavy stays, special handling, or clustered arrivals/departures.
- Payment/deposit/cancellation risk increases because unpaid deposits are attached to rooms that would otherwise be sellable during a high-demand window.

The trigger should not directly confirm/deny a reservation. It starts a deterministic evaluation that can produce availability decisions, staff tasks, review gates, manager alerts, waitlist recommendations, and customer-message drafts.

### Actors

- Customer/member: requests or modifies a boarding stay, may ask for dates, accommodation tier, add-ons, or cancellation/change exceptions.
- Pet(s): domain subjects whose species, care profile, temperament, medication, feeding, vaccine, and behavior facts affect eligibility and operational load.
- Front desk / customer care staff: review inquiries, explain approved policies, collect missing info, manage deposits, and place approved holds/updates in provider systems.
- Boarding staff / kennel team: fulfill care, arrival prep, room turnover, housekeeping, potty walks, play/enrichment, and shift handoffs.
- Shift lead: resolves day-of operational conflicts, ambiguous care evidence, staffing pressures, and room-turnover sequencing.
- Manager: approves capacity holds/overrides, waitlist priority exceptions, minimum-stay exceptions, deposit/cancellation waivers/refunds/forfeitures, staffing changes, and sensitive customer-facing explanations.
- AI/workflow agent: reads deterministic snapshots, drafts internal tasks, forecasts risk, drafts waitlist/follow-up language, and prepares manager packets. It never confirms availability, changes reservations, charges/refunds/waives money, or sends customer messages without typed approval.
- External provider/storage/tool adapters: reservation, availability, payment, document, messaging, and reporting systems. They execute only approved tool drafts.

### Inputs

- Location identity and policy version: `entities::LocationId`, local timezone, `LocationPolicyRefs`, holiday policy version.
- Requested stay facts: customer, pet(s), requested date range, accommodation preference/flexibility, requested add-ons, reservation source, and current reservation state if modifying.
- Season facts: named holiday period, demand class, blackout/manager-hold intervals, minimum-stay rule, deposit/cancellation overlays.
- Capacity facts: segmented inventory, booked counts, holds, pending deposits, waitlist queue, room-turnover constraints, late checkouts, maintenance/closed rooms.
- Care facts: species, care profile completeness, feeding/medication instructions, temperament/behavior/anxiety, group-play eligibility, vaccine/document status, belongings/report requirements.
- Payment facts: deposit rule, due timing, paid/unpaid status, collection draft status, refund/waiver/forfeit request, cancellation notice.
- Operational facts: staffing, arrival/departure volume, housekeeping load, open tasks, care exceptions, unresolved manager-review gates.
- Audit facts: actor, request source, workflow event, policy version, evidence snapshot identifiers, previous decisions.

### Decisions

1. Season classification: does the requested stay intersect a holiday/peak/blackout/local-event period, and which policy overlay applies?
2. Stay eligibility: does the requested date range satisfy the stricter of standard and holiday minimum-stay rules?
3. Accommodation segmentation: which typed segment is requested or acceptable: dog classic, dog luxury, cat condo, or flexible alternatives?
4. Capacity decision: can the location safely accept the stay for the segment/date range, should it waitlist, should it deny, or does it require manager review?
5. Deposit/payment decision: is a deposit required before confirmation, already paid, overdue, manager-review-only, or exception-requested?
6. Cancellation/change decision: does a change/cancellation during the holiday window map to no penalty, deposit forfeiture, manager review, or denial after check-in?
7. Care-load decision: do care profile gaps, medication, feeding exceptions, behavior, or play eligibility change the capacity posture or produce staff tasks?
8. Waitlist decision: if full, which waitlist lane applies and what typed reason/rank evidence is preserved?
9. Communication decision: which internal tasks and draft customer messages are safe to produce, and which review gates must be cleared before customer-facing delivery?
10. Staffing/handoff decision: does demand/care load require shift-lead or manager review for labor, room turnover, arrivals/departures, or care watchlists?

### Outputs

- `boarding::capacity::Decision`: `Available`, `Waitlist`, `Deny`, or `ManagerReview`, carrying typed reasons and evidence snapshot ids.
- `boarding::holiday::DemandSpikeAssessment`: named holiday period, affected segments, saturation, deposit exposure, care-load risk, and recommended posture.
- `boarding::stay::MinimumStay` selected by `boarding::season::Policy`, with `MinimumStayReason::HolidayPeak` or a more specific future reason.
- `boarding::deposit::Decision` and possibly a payment/deposit collection task draft.
- `boarding::cancellation::Outcome` for change/cancellation requests.
- `boarding::waitlist::EntryDraft` or `waitlist::Recommendation` for full/limited holiday inventory.
- `operations::StaffTask` drafts for missing deposit, care profile review, vaccine/document review, arrival prep, room turnover, medication/feeding review, manager capacity review, and customer follow-up.
- `workflow::RecommendedAction` values that preserve review gates and policy evidence.
- `operations::ResortDailyBrief` signals: boarding capacity risk, labor mismatch, pet-care watchlist, customer follow-ups, and `RevenueOpportunityKind::HolidayBoardingWaitlistFill`.
- Draft customer-message packets, never sent automatically.
- Audit event recording policy version, input snapshot ids, actor/agent, decision, review gate, and approved executor if any.

### Success state

A holiday-spike evaluation is successful when every affected stay/inquiry has a typed, auditable operational posture:

- Accepted stays have an approved accommodation segment, no overbooking, holiday minimum-stay compliance or manager-approved exception, required deposit status/collection task, and care/task plan.
- Full or unsafe segments produce waitlist/deny/manager-review decisions rather than false availability.
- Unpaid deposits and late cancellation/change requests are surfaced before they silently hold scarce holiday inventory.
- Staff have internal tasks and handoff packets for arrival/departure/care-load risks.
- Managers see capacity, labor, care, deposit, and waitlist risks in a daily brief or manager packet.
- Customer-facing drafts are review-gated and carry truthful policy evidence; no agent promises availability or takes payment action.

### Failure and exception states

- Missing or stale capacity snapshot: return `capacity::Decision::ManagerReview { reason: StaleOrMissingSnapshot }`; create internal task to refresh availability. Do not infer availability from old data.
- Unknown holiday policy for location/date: evaluate against conservative standard contract plus `ManagerReview { reason: MissingHolidayPolicy }`; do not skip minimum-stay/deposit checks.
- Accommodation ambiguity: produce `ManagerReview` or staff task when a request can flex between dog classic/luxury/cat segments only if species/accommodation compatibility and customer preference are unclear.
- Segment full: return `Waitlist` or `Deny` with `HolidaySegmentFull`; do not assign another segment without explicit customer/staff approval.
- Manager hold/blackout: return `ManagerReview` or `Deny` according to policy; only a manager-approved hold release can change the result.
- Deposit unpaid during peak window: block confirmation path with `deposit::Decision::Required`/`Overdue`; draft collection/follow-up tasks, not charges.
- Deposit waiver/refund/forfeit request: route to manager approval; never automated.
- Minimum-stay exception request: route to manager approval with typed reason and lost-capacity evidence.
- Late cancellation/change: compute typed outcome; sensitive explanations are draft-only until approval.
- Care or medical ambiguity: route to staff/manager/vet clarification; care risk can downgrade an otherwise available stay to review-required.
- Tool/provider disagreement: preserve both provider evidence and domain snapshot id, then route to review rather than selecting the optimistic answer.

## 2. Domain types to add or refine

Use semantic module paths under `operations::boarding`, adding child modules only where behavior needs ownership. Existing flat `operations::boarding::*` types can be kept as re-exports during migration.

### Season and holiday policy

- `operations::boarding::season::Period`
  - Fields: `id`, `location_id`, `name`, `date_range`, `demand_class`, `policy_version`.
  - Invariants: non-empty name; date range has positive duration; location/timezone explicit; policy version immutable for audit.
- `operations::boarding::season::DemandClass`
  - Variants: `Normal`, `Peak`, `Holiday`, `Blackout`, `LocalEvent`.
  - `Blackout` must never be treated as normal availability.
- `operations::boarding::season::Policy`
  - Owns mapping from stay date range to holiday overlays: minimum stay, deposit requirement, cancellation rule, capacity hold, waitlist posture.
- `operations::boarding::holiday::DemandSpikeAssessment`
  - Summary entity/value for a specific location + period + accommodation segment(s).
  - Invariants: every assessment cites a `capacity::SnapshotId` and `season::Period`; no customer-facing recommendation without approval state.
- `operations::boarding::holiday::SpikeTrigger`
  - `ForecastedDemand`, `InventoryThresholdCrossed`, `SegmentFull`, `LeadVolumeSurge`, `DepositExposure`, `CareLoadCluster`, `ManagerDeclaredPeak`.

### Stay and request

- `operations::boarding::stay::DateRange`
  - Check-in local date/time precedes checkout; derived `stay::Nights` >= 1.
- `operations::boarding::stay::Nights`
  - Positive scalar; can re-export current `StayNights` initially.
- `operations::boarding::stay::Request`
  - Required: location, customer, pets, date range, accommodation preference/flexibility, add-on requests, source.
  - It is a request, not proof of availability.
- `operations::boarding::stay::Plan`
  - Capacity/care/deposit-evaluated internal plan; still not an external confirmation until approval/tool execution.
- `operations::boarding::stay::MinimumStay`
  - Reuse current `MinimumStay`; ensure selected minimum is the strictest applicable policy.
- `operations::boarding::stay::MinimumStayReason`
  - Reuse current `MinimumStayReason::HolidayPeak`; consider adding `LocationHolidayPolicy`, `ManagerOverrideRequired`, and `BlackoutException` when code needs more precision.
- `operations::boarding::stay::MinimumStayDecision`
  - `Satisfied`, `Violation { required, requested }`, `ManagerReviewRequired { reason }`.

### Accommodation and capacity

- `operations::boarding::accommodation::Kind`
  - `Dog(DogSuiteKind)`, `Cat(CatAccommodationKind)`; prevents species/room leakage.
- `operations::boarding::accommodation::DogSuiteKind`
  - `ClassicSuite`, `LuxurySuite`.
- `operations::boarding::accommodation::CatAccommodationKind`
  - `CatCondo` initially.
- `operations::boarding::accommodation::Preference`
  - `Specific(Kind)`, `FlexibleWithinSpecies { preferred, acceptable }`, `StaffSelectWithinPolicy`.
- `operations::boarding::accommodation::Assignment`
  - Reservation/pet/date range/room id; cannot be constructed for incompatible species/accommodation.
- `operations::boarding::capacity::Inventory`
  - Segmented inventory by accommodation kind; each configured `RoomCount` positive.
- `operations::boarding::capacity::RoomCount`
  - Positive configured inventory count; current `RoomInventory` can re-export until segmented inventory lands.
- `operations::boarding::capacity::BookedCount`
  - Non-negative booked/held count.
- `operations::boarding::capacity::HoldCount`
  - Non-negative manager/policy holds distinct from booked rooms.
- `operations::boarding::capacity::SaturationBasisPoints`
  - 0..=10_000 normal occupancy; over-capacity must be explicit `OverbookedBy` instead of hidden as >100%.
- `operations::boarding::capacity::Availability`
  - Refine current `RoomAvailability` with `ManagerHold` in addition to `Open`, `Limited`, `WaitlistOnly`, `Closed`.
- `operations::boarding::capacity::Snapshot`
  - Location/date range/segment state with inventory, booked, holds, waitlist, stale/fresh marker, source ids.
- `operations::boarding::capacity::Decision`
  - `Available { assignment_policy, evidence }`, `Waitlist { reason, evidence }`, `Deny { reason, evidence }`, `ManagerReview { reason, evidence }`.
- `operations::boarding::capacity::WaitlistReason`
  - `HolidaySegmentFull`, `PolicyHold`, `PendingDepositRelease`, `CareLoadNeedsReview`, `StaleInventory`.
- `operations::boarding::capacity::DenialReason`
  - `Blackout`, `SpeciesAccommodationMismatch`, `MinimumStayViolationWithoutException`, `ClosedSegment`, `PolicyHardStop`.
- `operations::boarding::capacity::ReviewReason`
  - `ManagerHold`, `OverbookingRequested`, `StaleOrConflictingProviderData`, `HolidayExceptionRequested`, `CareLoadCluster`.

### Deposit, cancellation, and scarce-inventory exposure

- `operations::boarding::deposit::Rule`
  - Reuse current `DepositRule`; move/re-export once deposit behavior expands.
- `operations::boarding::deposit::Decision`
  - `NotRequired`, `Required { amount, due }`, `AlreadyPaid`, `Overdue { amount, due }`, `ManagerReviewRequired { reason }`.
- `operations::boarding::deposit::Exposure`
  - Holiday scarce-inventory risk from unpaid deposits holding rooms.
  - Invariant: references reservation/customer/period/segment and a payment status snapshot.
- `operations::boarding::deposit::ExceptionReason`
  - `WaiverRequested`, `RefundRequested`, `ForfeitRequested`, `ProviderMismatch`, `ComplaintOrServiceRecovery`.
- `operations::boarding::cancellation::Policy`
  - Reuse current `CancellationPolicy`; compose with season/deposit/check-in status.
- `operations::boarding::cancellation::Outcome`
  - `AllowedNoPenalty`, `AllowedForfeitDeposit`, `ManagerReviewRequired`, `DeniedAfterCheckIn`, `DraftCustomerExplanationRequired`.
- `operations::boarding::cancellation::NoticeWindow`
  - Uses `NoticeHours` and requested cancellation/change timestamp in location timezone.

### Waitlist and demand shaping

- `operations::boarding::waitlist::Entry`
  - Customer/pet/request/segment/period with rank evidence and status.
- `operations::boarding::waitlist::Status`
  - `Draft`, `StaffReviewed`, `Offered`, `Declined`, `Converted`, `Expired`, `Closed`.
- `operations::boarding::waitlist::Recommendation`
  - Internal recommendation to fill a released holiday room; not a customer offer until staff-approved.
- `operations::boarding::waitlist::PriorityPolicy`
  - Deterministic rank inputs; should not hide business choices in ad hoc sorting helpers.

### Care load, staff tasks, and handoff

- `operations::boarding::care::LoadAssessment`
  - Per stay/period/segment summary of medication, feeding, behavior, special handling, play eligibility, and profile gaps.
- `operations::boarding::care::LoadClass`
  - `Standard`, `Elevated`, `HighTouch`, `ManagerReviewRequired`.
- `operations::boarding::task::Kind`
  - Boarding-owned task kinds: `HolidayCapacityReview`, `DepositCollectionReview`, `WaitlistReview`, `ArrivalPrep`, `RoomTurnover`, `CareProfileReview`, `MedicationReview`, `FeedingReview`, `CustomerFollowUpDraft`, `ManagerExceptionReview`.
- `operations::boarding::handoff::Packet`
  - Holiday shift packet for capacity holds, late departures, arrivals, care-load clusters, unresolved deposits, waitlist releases, and manager gates.
- `operations::boarding::handoff::Severity`
  - `Info`, `NeedsShiftLead`, `NeedsManager`, `SafetyCritical`.

### Agent specs and approval

- `operations::boarding::agent::HolidayDemandSpikeSpec`
  - Agent/workflow spec for reading capacity/reservation/care/deposit snapshots and producing internal recommendations.
- `operations::boarding::agent::ApprovalPolicy`
  - Deterministic mapper from Boarding action to automation level/review gate.
- `operations::boarding::agent::ProposedAction`
  - `CreateInternalTask`, `DraftCustomerMessage`, `RecommendWaitlistOffer`, `RecommendCapacityHoldRelease`, `RecommendDepositFollowUp`, `RecommendManagerExceptionReview`, `SummarizeDailyBrief`.
- `operations::boarding::audit::DecisionRecord`
  - Immutable record of policy version, evidence ids, actor/agent, decision, review gate, and approval/execution state.

## 3. Relationship map between types

### Entities

- `entities::LocationId` selects `boarding::Contract`, `season::Policy`, capacity inventory, timezone, staff configuration, and provider adapters.
- `entities::CustomerId` anchors deposit exposure, waitlist entry, follow-up drafts, and customer-message review gates.
- `entities::PetId` anchors accommodation compatibility, care-load assessment, play eligibility, and staff tasks.
- `entities::ReservationId` links existing reservation state to boarding stay plans, deposit/cancellation outcomes, staff tasks, and tool drafts.
- `entities::Reservation` remains the cross-service lifecycle aggregate. Boarding owns the service-line decision that may recommend a reservation update; it should not mutate the reservation directly.

### Value objects

- `season::Period`, `stay::DateRange`, `stay::Nights`, `capacity::RoomCount`, `capacity::BookedCount`, `capacity::HoldCount`, `capacity::SaturationBasisPoints`, `deposit::Exposure`, and `audit::EvidenceRef` are immutable values used by policies and audit records.
- `accommodation::Kind` is the semantic join between species, inventory, room assignment, pricing/upgrade posture, and care obligations.
- `capacity::Snapshot` is a value snapshot, not a mutable room ledger. Assignment/hold changes flow through repositories/tool drafts.

### Policies

- `season::Policy` classifies dates and returns policy overlays.
- `stay::Policy` enforces holiday minimum stay and service-window constraints.
- `capacity::Policy` owns availability/waitlist/manager-review decisions from inventory and season evidence.
- `deposit::Policy` owns holiday deposit requirement/timing/exposure decisions.
- `cancellation::Policy` owns late holiday cancellation/change outcomes.
- `care::Policy` owns care-load review and task requirements.
- `waitlist::PriorityPolicy` owns waitlist recommendation rank.
- `agent::ApprovalPolicy` owns automation boundaries and review gates.

### Repositories and stores

- `operations::boarding::Repository`: contract/policy snapshot lookup by location/version.
- `operations::boarding::season::Repository`: holiday periods and policy overlays.
- `operations::boarding::capacity::Repository`: segmented snapshots, holds, provider evidence refs, and room assignment projections.
- `operations::boarding::reservation::Repository`: Boarding-facing reservation/date-range projections; returns semantic requests/plans, not raw provider rows.
- `operations::boarding::payment::Repository`: deposit/payment status evidence; drafts collection/waiver/refund/forfeit requests only through review-gated ports.
- `operations::boarding::care::Repository`: pet care/temperament/document/task evidence.
- `operations::boarding::waitlist::Repository`: waitlist entries/recommendations and status transitions.
- `operations::boarding::audit::Repository`: decision records and approval/execution trail.

### Workflow events

- `HolidayDemandSpikeDetected`
- `HolidayStayRequested`
- `HolidayCapacityThresholdCrossed`
- `HolidaySegmentFilled`
- `HolidayDepositExposureDetected`
- `HolidayWaitlistCandidateIdentified`
- `HolidayMinimumStayExceptionRequested`
- `HolidayCancellationOrChangeRequested`
- `HolidayCareLoadClusterDetected`
- `HolidayManagerReviewCleared`

These can be represented initially as existing `workflow::WorkflowEvent` values with Boarding-specific payloads, but the payload shape should preserve the semantic event name and evidence.

### Staff tasks

Boarding-owned task kinds map into generic `operations::StaffTaskKind` only at the boundary:

- `task::Kind::HolidayCapacityReview` -> `StaffTaskKind::CheckInPrep` or internal manager task with source detail until a richer generic kind exists.
- `task::Kind::DepositCollectionReview` -> `StaffTaskKind::CustomerFollowUp { reason: DepositNotPaid }` plus deposit evidence.
- `task::Kind::WaitlistReview` -> manager/front-desk task; should preserve `waitlist::Reason` and affected segment.
- `task::Kind::CareProfileReview` -> `StaffTaskKind::DocumentReview`, `MedicationAdministration`, `Feeding`, or `PlaygroupAssessment` depending on evidence.
- `task::Kind::RoomTurnover` -> `StaffTaskKind::CleaningTurnover`.

Do not force every Boarding nuance into generic `StaffTaskKind` variants. Boarding should own the semantic mapping and preserve source/reason in task metadata/audit.

### Agent specs and tools

- `boarding::agent::HolidayDemandSpikeSpec` reads reservations, capacity, season policy, care profile summaries, payment/deposit status, and existing waitlist entries.
- It emits `workflow::RecommendedAction`, `operations::StaffTask` drafts, `waitlist::Recommendation`, `deposit::FollowUpDraft`, and `operations::ResortDailyBrief` sections.
- It may call read-only availability/reservation/payment/care/document tools through typed ports.
- It may create internal task drafts if the approval policy marks them internal-only.
- It may not call live reservation confirmation, payment charge/refund/waiver, hold release, or customer-message send tools without an approved `AllowedAction`/review-clearance record.

## 4. Interaction contract

Rust-like pseudo-signatures show ownership, not exact implementation syntax.

### Stay request construction

```rust
impl boarding::stay::Request {
    pub fn builder() -> boarding::stay::request::Builder<MissingLocation, MissingCustomer, MissingPets, MissingDateRange>;

    pub fn accommodation_preference(&self) -> &boarding::accommodation::Preference;
    pub fn date_range(&self) -> boarding::stay::DateRange;
    pub fn pets(&self) -> &[entities::PetId];
}

impl boarding::stay::DateRange {
    pub fn new(
        check_in: location::LocalDateTime,
        check_out: location::LocalDateTime,
    ) -> boarding::stay::Result<Self>;

    pub fn nights(&self) -> boarding::stay::Nights;
    pub fn intersects(&self, period: &boarding::season::Period) -> bool;
}
```

Behavior belongs on `stay::DateRange` for date invariants and on `stay::Request` for request completeness. Capacity proof does not belong in the request builder.

### Season policy

```rust
pub trait boarding::season::Repository {
    async fn periods_for(
        &self,
        location: entities::LocationId,
        range: boarding::stay::DateRange,
    ) -> boarding::season::Result<Vec<boarding::season::Period>>;

    async fn policy_for(
        &self,
        location: entities::LocationId,
        version: boarding::season::PolicyVersion,
    ) -> boarding::season::Result<boarding::season::Policy>;
}

impl boarding::season::Policy {
    pub fn classify(&self, range: boarding::stay::DateRange) -> boarding::season::Classification;

    pub fn minimum_stay_for(
        &self,
        range: boarding::stay::DateRange,
    ) -> boarding::stay::MinimumStay;

    pub fn deposit_overlay_for(
        &self,
        period: &boarding::season::Period,
    ) -> boarding::deposit::Rule;

    pub fn cancellation_overlay_for(
        &self,
        period: &boarding::season::Period,
    ) -> boarding::cancellation::Policy;
}
```

Season policy owns holiday classification and overlays. The capacity policy should consume its output, not duplicate holiday date matching.

### Minimum-stay policy

```rust
impl boarding::stay::Policy {
    pub fn evaluate_minimum_stay(
        &self,
        request: &boarding::stay::Request,
        season: &boarding::season::Classification,
    ) -> boarding::stay::MinimumStayDecision;
}
```

Minimum-stay evaluation belongs to stay policy because it compares a stay request to stay requirements. Manager exception routing belongs to the decision, not to a helper boolean.

### Capacity policy

```rust
pub trait boarding::capacity::Repository {
    async fn snapshot_for(
        &self,
        location: entities::LocationId,
        range: boarding::stay::DateRange,
        segment: boarding::accommodation::Kind,
    ) -> boarding::capacity::Result<boarding::capacity::Snapshot>;

    async fn current_holds(
        &self,
        location: entities::LocationId,
        period: boarding::season::PeriodId,
    ) -> boarding::capacity::Result<Vec<boarding::capacity::Hold>>;

    async fn draft_hold_change(
        &self,
        draft: boarding::capacity::HoldChangeDraft,
    ) -> boarding::capacity::Result<workflow::RecommendedAction>;
}

impl boarding::capacity::Policy {
    pub fn evaluate(
        &self,
        request: &boarding::stay::Request,
        season: &boarding::season::Classification,
        snapshot: &boarding::capacity::Snapshot,
        care_load: &boarding::care::LoadAssessment,
    ) -> boarding::capacity::Decision;
}
```

Capacity policy owns availability/waitlist/manager-review decisions. It must not directly write provider reservations or send customer messages.

### Deposit policy

```rust
pub trait boarding::payment::Repository {
    async fn deposit_status_for(
        &self,
        reservation: entities::ReservationId,
    ) -> boarding::payment::Result<payment::DepositStatus>;

    async fn draft_deposit_collection(
        &self,
        request: boarding::deposit::CollectionDraft,
    ) -> boarding::payment::Result<workflow::RecommendedAction>;

    async fn draft_deposit_exception(
        &self,
        request: boarding::deposit::ExceptionDraft,
    ) -> boarding::payment::Result<workflow::RecommendedAction>;
}

impl boarding::deposit::Policy {
    pub fn decide(
        &self,
        request: &boarding::stay::Request,
        season: &boarding::season::Classification,
        reservation_deposit: Option<payment::DepositStatus>,
    ) -> boarding::deposit::Decision;

    pub fn exposure(
        &self,
        plan: &boarding::stay::Plan,
        status: payment::DepositStatus,
        snapshot: &boarding::capacity::Snapshot,
    ) -> Option<boarding::deposit::Exposure>;
}
```

Deposit policy owns requirement and exposure semantics. Payment tools own external transaction mechanics. Exceptions produce review-gated drafts only.

### Cancellation/change policy

```rust
impl boarding::cancellation::Policy {
    pub fn evaluate_change(
        &self,
        request: boarding::cancellation::ChangeRequest,
        season: &boarding::season::Classification,
        deposit: payment::DepositStatus,
        checked_in: boarding::stay::CheckedInState,
    ) -> boarding::cancellation::Outcome;
}
```

Cancellation/change decisions belong to cancellation policy and carry deposit/season evidence. Customer explanations are separate drafts gated by approval.

### Waitlist policy

```rust
pub trait boarding::waitlist::Repository {
    async fn entries_for(
        &self,
        location: entities::LocationId,
        period: boarding::season::PeriodId,
        segment: boarding::accommodation::Kind,
    ) -> boarding::waitlist::Result<Vec<boarding::waitlist::Entry>>;

    async fn save_draft(
        &self,
        draft: boarding::waitlist::EntryDraft,
    ) -> boarding::waitlist::Result<boarding::waitlist::EntryId>;
}

impl boarding::waitlist::PriorityPolicy {
    pub fn recommend_fill_order(
        &self,
        released_capacity: boarding::capacity::ReleasedCapacity,
        entries: &[boarding::waitlist::Entry],
    ) -> Vec<boarding::waitlist::Recommendation>;
}
```

Waitlist priority should be explicit policy. Do not encode ranking in anonymous sorting closures in the application layer.

### Care-load policy

```rust
pub trait boarding::care::Repository {
    async fn load_profile_summary(
        &self,
        pet: entities::PetId,
    ) -> boarding::care::Result<boarding::care::ProfileSummary>;

    async fn open_care_tasks_for(
        &self,
        reservation: entities::ReservationId,
    ) -> boarding::care::Result<Vec<operations::StaffTask>>;
}

impl boarding::care::Policy {
    pub fn assess_holiday_load(
        &self,
        request: &boarding::stay::Request,
        profiles: &[boarding::care::ProfileSummary],
        season: &boarding::season::Classification,
    ) -> boarding::care::LoadAssessment;

    pub fn task_requirements(
        &self,
        assessment: &boarding::care::LoadAssessment,
    ) -> Vec<boarding::task::Requirement>;
}
```

Care policy owns care-load risk. It can influence capacity decisions during holiday spikes because medication/feeding/behavior clusters affect staff capacity.

### Holiday demand-spike service

```rust
pub struct boarding::holiday::DemandSpikeService<C, S, R, P, Care, Audit> {
    capacity: C,
    season: S,
    reservation: R,
    payment: P,
    care: Care,
    audit: Audit,
    policies: boarding::holiday::Policies,
}

impl<C, S, R, P, Care, Audit> boarding::holiday::DemandSpikeService<C, S, R, P, Care, Audit> {
    pub async fn evaluate_request(
        &self,
        request: boarding::stay::Request,
        actor: workflow::ActorRef,
    ) -> boarding::holiday::Result<boarding::holiday::Evaluation>;

    pub async fn assess_period(
        &self,
        location: entities::LocationId,
        period: boarding::season::PeriodId,
    ) -> boarding::holiday::Result<boarding::holiday::DemandSpikeAssessment>;
}
```

The service coordinates repositories and policies. It should not hide domain decisions in free functions, and it should not perform live tool execution. Its output is an evaluation with explicit decisions, tasks, recommended actions, and audit records.

### Workflow planner

```rust
impl boarding::workflow::Planner {
    pub fn plan(
        &self,
        evaluation: boarding::holiday::Evaluation,
    ) -> boarding::workflow::Plan;
}

pub struct boarding::workflow::Plan {
    pub staff_tasks: Vec<operations::StaffTask>,
    pub recommended_actions: Vec<workflow::RecommendedAction>,
    pub customer_message_drafts: Vec<boarding::message::Draft>,
    pub daily_brief_sections: Vec<operations::DailyBriefSection>,
    pub review_gates: Vec<policy::ReviewGate>,
}
```

Planning belongs to Boarding workflow because it maps Boarding decisions into generic task/action envelopes while preserving source semantics.

### Agent approval policy

```rust
impl boarding::agent::ApprovalPolicy {
    pub fn classify(
        &self,
        action: boarding::agent::ProposedAction,
        evidence: &boarding::audit::DecisionRecord,
    ) -> policy::AutomationDecision;
}
```

Approval policy owns automation boundaries. Tool adapters should require an affirmative `AutomationDecision::Allowed`/approved review record before execution.

## 5. Review and approval contract

### Automation level

Safe internal automation:

- Read capacity, reservation, payment-status, care-summary, document-status, and waitlist snapshots.
- Classify holiday demand-risk from deterministic snapshots.
- Compute typed availability/waitlist/manager-review decisions.
- Draft internal staff tasks and manager daily brief sections.
- Draft customer follow-up language for missing info/deposit/waitlist, clearly marked unsent.
- Recommend waitlist fill candidates internally.
- Record decision/audit events for evaluations and drafts.

Staff review required:

- Any customer-facing message, waitlist offer, decline explanation, minimum-stay explanation, deposit reminder, or cancellation/change explanation.
- Applying an approved room assignment when capacity is limited or under holiday pressure.
- Clearing ambiguous care profile, feeding, medication, vaccine/document, behavior, or play eligibility facts.
- Marking free-text/media-based completion evidence as sufficient for holiday readiness.
- Converting a waitlist recommendation into an actual customer offer.

Manager approval required:

- Capacity override, overbooking, hold release, blackout exception, or holiday segment exception.
- Minimum-stay exception or accommodation substitution that materially changes the customer promise.
- Deposit waiver, refund, forfeit, exception, or payment dispute handling.
- Late holiday cancellation/change exception.
- Staffing schedule change, labor risk override, or manager-declared stop-sell/reopen decision.
- Sensitive customer-facing explanations involving denial, policy exception, money, incident, medical/care risk, safety, or complaint/service recovery.

Never fully automate:

- Confirming, cancelling, modifying, rejecting, or waitlist-offering a live reservation.
- Charging, refunding, waiving, or forfeiting money.
- Sending customer/member-facing messages.
- Final approval of medical, medication, behavior, incident, or safety decisions.
- Suppressing adverse facts from staff handoffs, reports, customer drafts, or audit trails.

### Review gates

Use typed review gates rather than prose-only warnings:

- `policy::ReviewGate::ManagerApproval` for capacity overrides, holds, minimum-stay exceptions, blackout exceptions, deposit exceptions, cancellation exceptions, and staffing changes.
- `policy::ReviewGate::CustomerMessageApproval` for all customer-facing draft sends.
- `policy::ReviewGate::MedicalDocumentReview` or a future `CareInstructionReview` for vaccine/medication/medical ambiguity.
- Future Boarding-specific gates may be useful: `HolidayCapacityReview`, `DepositExceptionReview`, `WaitlistOfferApproval`, `MinimumStayExceptionApproval`, `CareLoadReview`.

### Audit trail

Every holiday-spike decision record should include:

- Location, policy version, holiday period, timezone, and date range.
- Reservation/customer/pet identifiers if applicable.
- Accommodation segment and species compatibility evidence.
- Capacity snapshot id, provider evidence id, booked/held/waitlist counts, and staleness/freshness.
- Season/minimum-stay/deposit/cancellation policy inputs and selected overlay.
- Care-load assessment and unresolved care/document gates.
- Actor/agent identity, workflow event, proposed action, approval decision, reviewer identity if approved, and execution tool id if executed.
- Customer-message draft ids and sent status, if any.

Audit records must be durable and immutable enough that later policy changes do not rewrite why a holiday decision was made.

### Customer/member-facing boundaries

- Customer-visible copy must be generated as a draft tied to a review gate and source evidence.
- The system may explain internally why a request is waitlisted or needs review, but it must not send policy/legal/payment/safety explanations without approval.
- Availability language must avoid promises until capacity and approval are current. Prefer draft phrasing like “staff will review availability for these dates” over “we have a room.”
- Money language must not imply a charge/refund/waiver has occurred until payment tooling confirms an approved action.
- If care/behavior/medical facts are involved, drafts must be staff/manager-reviewed and must not diagnose, blame, or hide concerning facts.

## 6. Test contracts

Future Rust code cards should add semantic tests with names like these.

### Season and minimum stay

- `boarding_holiday_period_classification_uses_location_policy_version`
  - A date range intersecting a configured holiday returns the location-scoped holiday period and records the policy version.
- `boarding_holiday_minimum_stay_overrides_standard_minimum_when_stricter`
  - Holiday minimum stay wins over a shorter standard stay rule.
- `boarding_holiday_minimum_stay_violation_routes_to_manager_review_not_booking_confirmation`
  - A short holiday request yields `MinimumStayDecision::ManagerReviewRequired` or violation, not an available stay.
- `boarding_unknown_holiday_policy_routes_to_manager_review_with_conservative_default`
  - Missing policy data does not silently classify as normal demand.

### Capacity and accommodation segmentation

- `boarding_holiday_capacity_snapshot_preserves_classic_luxury_and_cat_segments`
  - Capacity snapshots keep accommodation kind as a semantic key.
- `boarding_holiday_capacity_waitlists_full_segment_without_cross_assigning_room_type`
  - A full dog classic segment cannot auto-assign cat condo or dog luxury without approved preference/offer flow.
- `boarding_holiday_capacity_manager_hold_returns_manager_review`
  - Manager-held capacity does not appear as open availability.
- `boarding_holiday_capacity_stale_snapshot_blocks_availability_promise`
  - Stale snapshot returns manager/staff review, not `Available`.
- `boarding_holiday_overbooking_request_requires_manager_approval`
  - Overbooking cannot be represented as normal saturation or normal availability.

### Deposit, cancellation, and scarce inventory

- `boarding_holiday_deposit_required_before_confirmation_when_due_at_booking`
  - Unpaid required holiday deposit blocks confirmation and creates a deposit follow-up draft/task.
- `boarding_holiday_unpaid_deposit_creates_capacity_exposure_for_manager_brief`
  - A scarce room held by unpaid deposit appears as `deposit::Exposure` and daily brief risk.
- `boarding_holiday_deposit_exception_cannot_be_automated`
  - Waiver/refund/forfeit requests map to manager review.
- `boarding_late_holiday_cancellation_uses_season_overlay_and_deposit_status`
  - Cancellation outcome composes notice, season, and deposit state.
- `boarding_holiday_change_request_rechecks_capacity_and_minimum_stay`
  - Date/accommodation changes do not reuse stale approval from the original stay.

### Care load and handoff

- `boarding_holiday_care_load_cluster_can_escalate_capacity_decision_to_review`
  - Medication/feeding/behavior clusters influence review even when room count is available.
- `boarding_holiday_missing_medication_instruction_creates_staff_review_task`
  - Medication ambiguity creates a care task/review gate.
- `boarding_holiday_behavior_flag_suppresses_unsupervised_play_or_upsell_recommendation`
  - Behavior/anxiety risk blocks unsafe enrichment/upsell suggestions.
- `boarding_holiday_handoff_packet_includes_capacity_holds_deposit_exposure_and_care_load_risks`
  - Shift handoff preserves holiday-specific obligations.
- `boarding_holiday_room_turnover_tasks_reflect_arrival_departure_clustering`
  - Arrival/departure clusters create room-turnover tasks with due times and priority.

### Waitlist and demand shaping

- `boarding_holiday_waitlist_entry_preserves_segment_period_and_rank_reason`
  - Waitlist entries are not plain customer lists; they carry segment/period/reason evidence.
- `boarding_holiday_waitlist_fill_recommendation_remains_internal_until_staff_approved`
  - Released capacity creates internal recommendation only.
- `boarding_holiday_waitlist_priority_policy_is_deterministic_and_auditable`
  - Same evidence produces same rank order and audit explanation.

### Workflow, agents, and approval boundaries

- `boarding_holiday_agent_can_create_internal_capacity_review_task_but_cannot_confirm_reservation`
  - Approval policy permits internal task drafting and denies live confirmation.
- `boarding_holiday_agent_cannot_send_waitlist_offer_without_customer_message_approval`
  - Waitlist customer offer remains draft-only.
- `boarding_holiday_agent_cannot_charge_refund_waive_or_forfeit_deposit`
  - Payment actions require approval and tool boundary.
- `boarding_holiday_manager_daily_brief_surfaces_capacity_labor_care_deposit_and_waitlist_risks`
  - Daily brief carries all risk classes with Boarding context.
- `boarding_holiday_audit_record_preserves_policy_snapshot_and_approval_chain`
  - Decision record includes policy version, evidence ids, proposed action, review gate, reviewer, and execution id.

### Storage and serialization

- `boarding_holiday_policy_snapshot_roundtrips_without_losing_demand_class_or_period_name`
  - Storage codecs preserve holiday period semantics.
- `boarding_holiday_capacity_snapshot_roundtrips_segmented_inventory_and_holds`
  - Storage keeps booked/held/waitlist counts separate.
- `boarding_existing_standard_petsuites_contract_remains_serialization_compatible_during_reexport_migration`
  - Refactoring from flat types to child modules does not break current `CoreServiceContracts` storage unexpectedly.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Near-term: extend `operations::boarding` with child modules or carefully staged nested types for `season`, `stay`, `accommodation`, `capacity`, `deposit`, `cancellation`, `care`, `waitlist`, `holiday`, `workflow`, `agent`, and `audit`.
  - Consider splitting `operations.rs` into `operations/mod.rs` plus service-line modules only if the code card scope includes module migration and storage compatibility tests.
- `domain/src/entities.rs`
  - Likely no new ownership, but `Reservation`/`ServiceKind::Boarding` projections may need typed references in tests.
- `domain/src/payment.rs` / `domain/src/money.rs`
  - Reuse existing money/deposit primitives. Do not duplicate actual transaction state in Boarding.
- `domain/src/policy.rs`
  - Add Boarding-specific review gates only if existing generic gates are insufficient.
- `domain/src/workflow.rs`
  - Add or reuse workflow event/action envelopes for Boarding holiday evaluations and audit records.
- `domain/src/tools.rs`
  - Extend availability/payment/reservation draft ports only as approved drafts, not live mutators.
- `domain/src/agents.rs`
  - Add or specialize a holiday demand/capacity alert agent spec if baseline `booking-triage` and `manager-daily-brief` are too broad.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add holiday-focused contract tests while keeping existing standard Boarding contract tests green.
- `domain/tests/domain_quality_patterns.rs`
  - Add semantic path/call-site tests if doctrine checks cover module path preservation.
- `domain/tests/storage_*` or storage crate tests if contract snapshots are persisted outside `domain/tests`.

### Migration/refactor risks

- Current Boarding types are flat under `operations::boarding`. Moving them to child modules may break serialization paths, derives, imports, and storage roundtrips. Prefer staged re-exports:
  - `pub mod stay { pub use super::StayNights as Nights; ... }` only as a temporary compatibility bridge, or define new child-module types with explicit conversions.
  - Preserve `operations::boarding::Contract::standard_petsuites()` behavior until storage migrations are explicit.
- Existing `RoomInventory` is a single positive count. Holiday spikes require segmented inventory. Do not overload one count with mixed classic/luxury/cat capacity.
- Existing `RoomAvailability` lacks `ManagerHold`. Adding enum variants can affect serialization compatibility; migration needs explicit default/unknown handling if persisted data exists.
- Existing `DepositRule::Required { amount }` does not encode due timing or exception state. Do not treat it as actual payment/deposit status; compose with `payment::DepositStatus`.
- Existing `PaymentTiming` is contract-level. Holiday deposit due-at-booking behavior may need a deposit-specific due value rather than reusing generic checkout timing.
- Generic `tools::AvailabilityResult` has only `Available`/`Unavailable`. Boarding capacity decisions need `Waitlist` and `ManagerReview`; adapt carefully at the boundary rather than collapsing semantics.
- Generic `operations::StaffTaskKind` may be too coarse. Preserve Boarding task reason/source metadata so holiday semantics are not lost.
- If chrono local dates/timezones are not modeled yet, holiday periods can be accidentally evaluated in UTC. Use location-local date/time semantics for check-in/check-out and policy deadlines.
- Avoid typestate until behavior truly needs compile-time phase legality. Runtime semantic enums may be enough for the first holiday implementation slice.

### Dependencies on other implications/cards

- Depends on the Boarding service-domain map as the canonical vocabulary and approval boundary.
- Strong dependency on accommodation/capacity segmentation. Holiday demand cannot be safely modeled while capacity is one undifferentiated room count.
- Depends on season/holiday policy data being location-scoped and versioned.
- Depends on deposit/payment status integration for scarce-inventory exposure and before-confirmation gates.
- Depends on care-profile and temperament projections for care-load clusters during peak dates.
- Depends on workflow/audit envelopes before agents can produce trustworthy manager packets.
- Later Pawgress/report implications should consume the same approval-boundary pattern: drafts are safe; sends are gated.

### Recommended implementation slice order

1. Add `accommodation::Kind`, segmented `capacity::Inventory/Snapshot`, and `capacity::Decision` with tests for species/accommodation compatibility and full holiday segment waitlisting.
2. Add `season::Period/Policy` and `stay::MinimumStayDecision`, preserving existing `MinimumStayReason::HolidayPeak` and storage compatibility.
3. Add `deposit::Decision/Exposure` and cancellation outcome composition with manager-review gates.
4. Add `care::LoadAssessment` and Boarding-owned `task::Kind` mapping to generic staff tasks.
5. Add `waitlist::Entry/Recommendation` and priority policy as internal-only workflow outputs.
6. Add `holiday::DemandSpikeService` orchestration and `agent::ApprovalPolicy` tests.
7. Add storage/serialization roundtrips for policy snapshots, segmented capacity snapshots, waitlist entries, and audit records.

### Non-goals for the first code card

- Live provider booking/confirmation or payment execution.
- Dynamic pricing or revenue management.
- Customer-message sending.
- Full room-assignment optimization.
- Broad `operations.rs` module surgery without storage compatibility protection.
- Replacing generic reservation/customer/pet/payment ownership with Boarding-local duplicates.
