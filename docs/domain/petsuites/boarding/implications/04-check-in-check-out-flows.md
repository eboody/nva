# Boarding implication 04: check-in/check-out flows

Purpose: model the Boarding operational implication for arrival and departure workflows. This is a domain-contract artifact for later Rust implementation. It does not authorize live reservation status changes, payment collection, customer messaging, Pawgress Report sending, or room reassignment.

Source context:

- `docs/domain/petsuites/boarding/service-domain-map.md`
- `domain/src/operations.rs`
- `domain/src/entities.rs`
- `domain/src/workflow.rs`
- `domain/src/tools.rs`
- `domain/tests/petsuites_core_service_contracts.rs`

Assumptions:

- Check-in/check-out windows, fees, room identifiers, deposit deadlines, and late-departure rules are location-policy data. The domain core should model them as typed policy inputs, not hard-coded PetSuites constants.
- Boarding check-in/check-out are staff-operated workflows over an existing `entities::Reservation`; they may recommend reservation status changes but should not mutate live provider systems directly from agent output.
- A stay can involve multiple pets. Arrival/departure readiness must be representable per reservation and per pet because care, belongings, medication, play, and Pawgress Report evidence may differ by pet.
- Unknown business details should default to the safest extensible model: create internal review tasks, require explicit staff/manager approval, and preserve an audit trail.

## 1. Operational story

### Check-in trigger

A check-in flow begins when one of these typed triggers is observed:

- the reservation enters the location's arrival preparation window;
- a customer/pet physically arrives at the resort;
- a staff member opens an arrival workflow for a confirmed or offered boarding stay;
- an agent detects that a same-day boarding arrival has missing profile, care, payment, room-assignment, or document requirements and drafts internal tasks.

The trigger should be modeled as `operations::boarding::arrival::Trigger`, not as a free-text note. Suggested variants:

```rust
pub enum operations::boarding::arrival::Trigger {
    ArrivalPrepWindowOpened { reservation_id: entities::ReservationId },
    CustomerArrived { reservation_id: entities::ReservationId, actor: entities::ActorRef },
    StaffOpenedArrival { reservation_id: entities::ReservationId, actor: entities::ActorRef },
    AgentDetectedReadinessGap { reservation_id: entities::ReservationId, event_id: workflow::WorkflowEventId },
}
```

### Check-in actors

- Front desk staff verifies identity, reservation status, required profile/document/payment state, contact preferences, belongings, and customer-facing expectations.
- Kennel technician or lead staff verifies per-pet care instructions, medication, feeding, behavior, accommodation assignment, and first care tasks.
- Manager approves exceptions: capacity override, missing/ambiguous care facts, deposit waiver, late policy exception, special handling, behavior/medical risk, or customer complaint.
- Agent may summarize, detect gaps, draft check-in tasks, draft workflow events, and recommend review gates. It must not confirm arrival, collect/waive money, approve medical/behavior ambiguity, assign limited inventory, or send customer messages without typed approval.
- Customer/member is a source of arrival information and belongings but should not be treated as an actor that completes internal review gates unless the event is modeled as incoming evidence requiring staff validation.

### Check-in inputs

Inputs should be loaded through truthful repositories/ports:

- `entities::Reservation`: reservation ID, location, customer, pet IDs, service kind, status, add-ons, hard stops, deposit/payment state.
- `entities::Customer`: identity, preferred contact channel, portal/account reference.
- `entities::Pet`: species, care profile, temperament, medication instructions, vaccine/profile evidence, spay/neuter/group-play facts where available.
- `operations::boarding::Contract`: location-scoped arrival window, capacity posture, minimum-stay/deposit/cancellation defaults, housekeeping/handoff requirements.
- `operations::boarding::capacity::Snapshot` and `accommodation::Assignment`: room segment and concrete assignment evidence.
- `payment::Deposit` / `payment::DepositStatus`: actual deposit/payment state; Boarding owns policy interpretation but not transaction truth.
- `operations::StaffTask` evidence: existing open/prep/document/care tasks.
- `workflow::WorkflowEvent` and audit events: previously recorded decisions/reviews.

### Check-in decisions

A check-in evaluation should return a typed `arrival::Decision`, not a boolean `can_check_in`:

```rust
pub enum operations::boarding::arrival::Decision {
    ReadyToCheckIn { packet: arrival::Packet, tasks: Vec<operations::StaffTask> },
    NeedsStaffReview { packet: arrival::Packet, reasons: Vec<arrival::ReviewReason>, tasks: Vec<operations::StaffTask> },
    NeedsManagerApproval { packet: arrival::Packet, gates: Vec<policy::ReviewGate>, tasks: Vec<operations::StaffTask> },
    Blocked { packet: arrival::Packet, blocks: Vec<arrival::BlockReason>, tasks: Vec<operations::StaffTask> },
}
```

Decision points:

- Is the reservation in a status that can enter arrival workflow (`Confirmed`, possibly `Offered` with staff action) rather than `Cancelled`, `Rejected`, or unresolved `MissingInfo`?
- Is the customer identity/profile attached and fit for local policy?
- Are all pets species/accommodation-compatible, and is each room assignment valid for the booked stay?
- Are care instructions complete enough for staff to fulfill the first night: feeding, medication, allergies, behavior/special handling, emergency/vet contact, and add-on requirements?
- Are required vaccination/profile documents present or routed to document review?
- Is deposit/payment status acceptable for the contract/payment timing?
- Are requested add-ons and playtime options eligible or review-gated?
- Are belongings captured and labeled with a reviewable manifest?
- Do any medical, behavior, incident, missing evidence, or customer complaint signals require lead/manager review before check-in completion?

### Check-in outputs

The workflow should produce a durable `arrival::Packet` plus recommendations, never silent side effects:

- `arrival::Packet`: normalized arrival facts, per-pet readiness, room assignment evidence, belongings manifest, review gates, and audit references.
- `arrival::Checklist`: typed checklist items and completion/review state.
- `operations::StaffTask` drafts: document review, feeding clarification, medication double-check, behavior review, room prep, belongings intake, payment follow-up, Pawgress setup, playtime assessment.
- `workflow::RecommendedAction::UpdateStatus`: status suggestion to `CheckedIn` or `SpecialReview`, carrying a typed reason. The provider update remains a tool-bound approved action.
- `workflow::WorkflowEvent`: policy context, allowed actions, review gates, actor, and subject.
- `entities::AuditEvent`: policy decisions, suggested/resolved status changes, and approvals.

### Check-in success state

A successful check-in means:

- reservation status is approved to transition to `entities::ReservationStatus::CheckedIn` through a reviewed/allowed update path;
- every pet has a compatible accommodation assignment and initial care plan;
- deposit/payment preconditions are satisfied or explicitly review-approved;
- belongings and customer-provided evidence are captured;
- open care tasks exist for medication/feeding/play/housekeeping/Pawgress obligations;
- unresolved exceptions are either absent or represented as open staff/manager tasks and not hidden from handoff.

### Check-in failure and exception states

- `arrival::BlockReason::ReservationStatusNotArrivalEligible`: cancelled/rejected/missing-info reservations cannot be checked in.
- `arrival::BlockReason::AccommodationMismatch`: cat assigned to dog suite, dog assigned to cat condo, or room segment mismatch.
- `arrival::BlockReason::NoRoomAssignment`: no valid accommodation assignment and capacity policy does not allow automatic assignment.
- `arrival::BlockReason::PaymentOrDepositRequired`: deposit/payment is required before or at check-in and is not satisfied.
- `arrival::BlockReason::MissingRequiredDocument`: vaccine/profile/document proof missing.
- `arrival::BlockReason::CareInstructionsIncomplete`: feeding, medication, allergy, medical, emergency, or behavior facts are missing/ambiguous.
- `arrival::BlockReason::ManagerApprovalRequired`: capacity/holiday/deposit/care/incident/customer complaint exception.
- `arrival::BlockReason::ProviderSyncUnverified`: provider/system state cannot be verified; fail closed into staff review.

### Check-out trigger

A check-out flow begins when:

- the reservation enters the local departure preparation window;
- the customer arrives for pickup;
- staff opens departure workflow;
- the stay reaches the scheduled check-out date/time and open care/payment/report/belongings tasks remain;
- an agent detects late-departure/payment/report/belongings risk and drafts internal tasks.

Model as `operations::boarding::departure::Trigger` with variants mirroring arrival and carrying reservation/actor/event evidence.

### Check-out actors

- Front desk staff verifies final invoice/payment state, customer pickup, belongings return, customer-visible report/message, and reservation status suggestion.
- Kennel technician/lead staff verifies final care tasks, room turnover readiness, Pawgress Report evidence, and any incident/medical/behavior handoff.
- Groomer/trainer may contribute completion evidence when exit bath, grooming, or training add-ons are attached.
- Manager approves payment/deposit exceptions, sensitive report/message content, incidents, unresolved care gaps, late-departure exceptions, refunds/waivers/forfeitures, and complaints.
- Agent may summarize open tasks, draft checkout packets, draft Pawgress Report text from approved evidence, recommend exit bath/grooming/training follow-up internally, and flag late-departure/payment risk. It may not charge/refund money, mark sensitive reports sent, complete checkout, or send customer messages without approval.

### Check-out inputs

- `entities::Reservation`: status, scheduled departure, pets, add-ons, hard stops, service kind.
- `operations::boarding::departure::ReadinessPacket`: current checkout state if already started.
- `operations::StaffTask`: open/completed care tasks and evidence.
- `operations::boarding::care::Plan`: per-pet care obligations and unresolved review gates.
- `operations::boarding::report::PawgressReport`: draft/review/approved/sent state and source evidence.
- `payment::DepositStatus`, payment references, invoice/payment summary projection.
- `operations::boarding::belongings::Manifest`: items checked in and return state.
- `operations::boarding::upsell::Recommendation`: exit bath/grooming/training/rebooking opportunities and suppression rationale.
- `operations::boarding::capacity::Repository` or room store: room turnover / cleaning readiness projection.

### Check-out decisions

Return a typed `departure::Decision`:

```rust
pub enum operations::boarding::departure::Decision {
    ReadyToCheckOut { packet: departure::Packet, tasks: Vec<operations::StaffTask> },
    NeedsStaffReview { packet: departure::Packet, reasons: Vec<departure::ReviewReason>, tasks: Vec<operations::StaffTask> },
    NeedsManagerApproval { packet: departure::Packet, gates: Vec<policy::ReviewGate>, tasks: Vec<operations::StaffTask> },
    Blocked { packet: departure::Packet, blocks: Vec<departure::BlockReason>, tasks: Vec<operations::StaffTask> },
}
```

Decision points:

- Is the reservation currently checked in/active and eligible for departure workflow?
- Are all required care tasks complete or explicitly waived/closed by the correct actor?
- Are medication/feeding/incident/behavior notes resolved enough to release the pet and communicate truthfully?
- Is payment due at checkout satisfied, or is a reviewed payment task/gate present?
- Has the belongings manifest been reconciled per pet/reservation?
- Is the Pawgress Report either not required, approved to send, sent, or intentionally suppressed with a review reason?
- Are add-on tasks complete: exit bath, grooming, training, playtime, premium bedding return, photo/video update?
- Does late pickup, extended checkout, incident, complaint, refund/deposit exception, or unresolved care gap require manager approval?
- Should a room turnover/cleaning task be emitted before the room returns to inventory?

### Check-out outputs

- `departure::Packet`: final release packet with per-pet release readiness, payment state, report state, belongings reconciliation, room turnover requirement, review gates, and audit refs.
- `workflow::RecommendedAction::UpdateStatus`: status suggestion to `CheckedOut`, `Active` with late-departure risk, or `SpecialReview` depending on policy.
- `operations::StaffTask` drafts: checkout prep, belongings review, payment follow-up, report review, room turnover, incident follow-up, customer follow-up, exit bath/grooming/training follow-up.
- `boarding::report::PawgressReport` state transition drafts, not direct sends.
- `boarding::upsell::Recommendation` values for staff review, not customer-facing offers.
- `entities::AuditEvent` for release decision, approvals, suppressed report/recommendation rationale, and payment/exception review.

### Check-out success state

A successful check-out means:

- the reservation is approved to transition to `entities::ReservationStatus::CheckedOut` through a reviewed/allowed provider update;
- all pets have release readiness complete;
- required care tasks and add-on tasks are completed or explicitly review-resolved;
- payment/deposit obligations are satisfied or review-approved;
- belongings are returned/reconciled;
- report/message actions are approved/sent or explicitly withheld with a review reason;
- room turnover/housekeeping task exists before capacity returns the accommodation to sellable inventory;
- audit trail connects final status, actor, evidence, policy context, and review gates.

### Check-out failure and exception states

- `departure::BlockReason::ReservationNotCheckedIn`: cannot complete checkout for a stay that never entered checked-in/active state.
- `departure::BlockReason::OpenCareTask`: required feeding/medication/play/incident/care task unresolved.
- `departure::BlockReason::PaymentDue`: payment due at checkout is unpaid and no approved exception exists.
- `departure::BlockReason::BelongingsUnreconciled`: manifest mismatch or missing customer item.
- `departure::BlockReason::ReportRequiresReview`: Pawgress/customer message has sensitive content or lacks approval.
- `departure::BlockReason::IncidentOrMedicalReviewOpen`: release requires lead/manager review.
- `departure::BlockReason::LateDepartureException`: extended checkout or late pickup needs policy/manager handling.
- `departure::BlockReason::ProviderSyncUnverified`: provider status/payment/task sync cannot be verified.

## 2. Domain types to add/refine

### Arrival/check-in modules

- `operations::boarding::arrival::Trigger`
  - Closed enum of check-in workflow causes. Invariant: carries a reservation ID and either staff/customer/agent evidence.
- `operations::boarding::arrival::Packet`
  - Aggregate of arrival readiness evidence. Invariant: one location and one reservation; all pet-level entries belong to that reservation.
- `operations::boarding::arrival::Checklist`
  - Ordered, typed checklist with item states. Invariant: no duplicate semantic item kind for the same pet/reservation scope unless the item kind is explicitly repeatable.
- `operations::boarding::arrival::ChecklistItem`
  - `ProfileVerified`, `DocumentsVerified`, `DepositReviewed`, `AccommodationAssigned`, `BelongingsReceived`, `FeedingReviewed`, `MedicationReviewed`, `BehaviorReviewed`, `AddOnsReviewed`, `InitialCareTasksCreated`, `CustomerExpectationsReviewed`.
- `operations::boarding::arrival::ItemState`
  - `NotStarted`, `Satisfied { evidence }`, `NeedsStaffReview { reason }`, `NeedsManagerApproval { gate }`, `Blocked { reason }`, `NotApplicable { reason }`.
- `operations::boarding::arrival::Decision`
  - Semantic decision enum listed above. Invariant: blocked decisions carry at least one block; review decisions carry at least one reason/gate.
- `operations::boarding::arrival::ReviewReason`
  - Missing/ambiguous care, document, payment, accommodation, behavior, add-on, customer communication, or provider-sync review.
- `operations::boarding::arrival::BlockReason`
  - Hard blockers listed in the story.
- `operations::boarding::arrival::Policy`
  - Owns readiness evaluation. Invariant: does not execute provider status changes; it returns decisions and drafts.
- `operations::boarding::arrival::Planner`
  - Converts an arrival decision into staff tasks and recommended workflow actions.

### Departure/check-out modules

- `operations::boarding::departure::Trigger`
  - Closed enum of departure workflow causes.
- `operations::boarding::departure::Packet`
  - Aggregate of departure readiness evidence. Invariant: cannot be `ReadyToCheckOut` if required payment, belongings, care, or report gates remain unresolved.
- `operations::boarding::departure::Checklist`
  - Typed checklist for final care, add-ons, payment, report, belongings, release, and room turnover.
- `operations::boarding::departure::ChecklistItem`
  - `FinalCareCompleted`, `MedicationClosed`, `IncidentReviewClosed`, `PaymentReviewed`, `BelongingsReturned`, `PawgressReportResolved`, `ExitBathOrGroomingCompleted`, `UpsellRecommendationsReviewed`, `PetReleased`, `RoomTurnoverCreated`.
- `operations::boarding::departure::ItemState`
  - Same semantic shape as arrival, but with departure-specific reasons.
- `operations::boarding::departure::Decision`
  - Semantic decision enum listed above.
- `operations::boarding::departure::ReviewReason`
  - Open care evidence, payment, belongings, report, incident, late pickup, room turnover, customer follow-up.
- `operations::boarding::departure::BlockReason`
  - Hard blockers listed in the story.
- `operations::boarding::departure::Policy`
  - Owns checkout-readiness evaluation.
- `operations::boarding::departure::Planner`
  - Converts departure decisions into staff tasks/recommended actions.

### Shared check-in/check-out concepts

- `operations::boarding::stay::Phase`
  - `Booked`, `ArrivalPrep`, `CheckedIn`, `ActiveStay`, `DeparturePrep`, `CheckedOut`, `ExceptionReview`. Invariant: this is a Boarding projection, not a replacement for `entities::ReservationStatus`.
- `operations::boarding::stay::PhaseTransition`
  - Suggested phase transition with reason, actor, event/audit refs, and review gate state.
- `operations::boarding::readiness::Scope`
  - `Reservation`, `Pet(entities::PetId)`, `Accommodation(accommodation::RoomId)`, `Customer(entities::CustomerId)`, `Payment`, `Report`.
- `operations::boarding::readiness::EvidenceRef`
  - Typed reference to staff task evidence, workflow event, audit event, provider external ID, payment reference, document/media ID, or customer-provided note. Invariant: references are non-empty and source-typed.
- `operations::boarding::belongings::Manifest`
  - Reservation-level belongings intake/release record. Invariant: item labels are non-empty; checked-out manifest cannot claim completion while required items remain `Missing`, `Damaged`, or `CustomerDeclined` without review.
- `operations::boarding::belongings::Item`
  - `name`, optional pet association, intake condition, release condition, evidence.
- `operations::boarding::belongings::ItemState`
  - `Received`, `Stored`, `Returned`, `Missing`, `Damaged`, `CustomerDeclined`, `NotApplicable`.
- `operations::boarding::care::Readiness`
  - Per-pet care readiness summary: feeding, medication, behavior, allergy/medical, playtime, add-on, report evidence.
- `operations::boarding::payment::Readiness`
  - Boarding interpretation of payment/deposit status. Invariant: uses `payment` values for transaction truth; Boarding only adds policy outcome/review gate.
- `operations::boarding::report::Resolution`
  - `NotRequired`, `DraftNeeded`, `Drafted`, `StaffReviewRequired`, `ManagerReviewRequired`, `ApprovedToSend`, `Sent`, `Suppressed { reason }`.
- `operations::boarding::task::Kind`
  - Boarding-owned staff task kinds: `ArrivalPrep`, `ArrivalDocumentReview`, `ArrivalCareReview`, `MedicationDoubleCheck`, `BelongingsIntake`, `RoomPrep`, `DeparturePrep`, `FinalCareReview`, `BelongingsReturn`, `PaymentFollowUp`, `ReportReview`, `RoomTurnover`, `LateDepartureReview`, `CustomerFollowUp`.
- `operations::boarding::workflow::EventKind`
  - Boarding-specific event labels that map into `workflow::WorkflowEventType`/audit extension labels without overloading generic workflow enums prematurely.

### Refinements to current types

- Keep `operations::boarding::Contract` as the root contract and add arrival/departure policy fields only when behavior needs them. Current `arrival_window`, `departure_window`, `PaymentTiming`, `HandoffRequirement`, and `HousekeepingCadence` already anchor this implication.
- Refactor `operations::boarding::HandoffRequirement` into `handoff::Requirement` or compose it with `arrival::ChecklistItem` / `departure::ChecklistItem` once multiple requirements can apply simultaneously.
- Refactor generic `operations::StaffTaskKind::{CheckInPrep, CheckOutPrep}` to preserve Boarding task meaning by carrying a `boarding::task::Kind` source projection or adding a domain-owned mapper.
- Preserve `entities::ReservationStatus::{CheckedIn, Active, CheckedOut}` as the cross-service lifecycle states; Boarding phase/readiness should not become a parallel reservation aggregate.
- Preserve `workflow::RecommendedAction::UpdateStatus` and `tools::ReservationUpdateDraft` as status-change boundaries.

## 3. Relationship map between types

### Entities

- `entities::Reservation`
  - Cross-service aggregate. Boarding arrival/departure policies evaluate it and produce suggested status updates.
- `entities::Customer`
  - Provides identity/contact facts; does not own check-in/check-out rules.
- `entities::Pet`
  - Provides species/care/temperament facts consumed by Boarding care readiness.
- `entities::Location`
  - Selects Boarding contract/policy version and local windows.
- `entities::AuditEvent`
  - Durable audit record for policy decisions, status suggestions/changes, approvals, report/message boundaries, and payment exceptions.

### Boarding entities / aggregates

- `operations::boarding::Contract`
  - Root service-line contract; composes arrival/departure policy defaults, stay/payment/handoff/care posture.
- `operations::boarding::arrival::Packet`
  - Arrival-readiness aggregate for one reservation.
- `operations::boarding::departure::Packet`
  - Departure-readiness aggregate for one reservation.
- `operations::boarding::belongings::Manifest`
  - Owned by Boarding stay workflow because it is operational arrival/departure evidence.
- `operations::boarding::report::PawgressReport`
  - Customer-facing artifact derived from staff evidence, approval state, and report policy.

### Value objects

- `operations::boarding::stay::DateRange`, `stay::Nights`, `service_window::Window`, `service_window::HourOfDay`.
- `operations::boarding::accommodation::RoomId`, `accommodation::Assignment`, `accommodation::Kind`.
- `operations::boarding::readiness::EvidenceRef`, `readiness::Scope`.
- `operations::boarding::belongings::Item`, `belongings::ItemState`.
- `operations::boarding::payment::Readiness` and `report::Resolution`.

### Policies

- `arrival::Policy` evaluates readiness to check in.
- `departure::Policy` evaluates readiness to check out.
- `care::Policy` evaluates feeding/medication/behavior/play/add-on readiness.
- `deposit::Policy` evaluates deposit requirements; actual money state comes from `payment`.
- `capacity::Policy` validates room/accommodation assignment and limited-inventory manager review.
- `report::ApprovalPolicy` gates Pawgress and customer-visible report text.
- `agent::ApprovalPolicy` maps agent-produced arrival/departure drafts to automation levels and review gates.

### Repositories / stores

- `operations::boarding::Repository`: contract/policy snapshots by location.
- `operations::boarding::reservation::Repository`: Boarding-facing reservation projection and status-draft port.
- `operations::boarding::arrival::Repository`: persists arrival packets/checklists and review state.
- `operations::boarding::departure::Repository`: persists departure packets/checklists and release state.
- `operations::boarding::care::Repository`: reads care profile/task evidence and writes internal care task drafts/projections.
- `operations::boarding::capacity::Repository`: reads room assignment/capacity snapshots and writes approved internal holds/turnover projections.
- `operations::boarding::payment::Repository`: reads deposit/payment state and drafts payment follow-up/exception actions.
- `operations::boarding::report::Repository`: stores report drafts, source evidence, approval state, and sent metadata.
- `storage::operations`: storage adapters convert records to/from semantic domain values; provider JSON stays at the boundary.

### Workflow events

- Arrival: `ArrivalPrepWindowOpened`, `CheckInStarted`, `CheckInReadinessEvaluated`, `CheckInReviewRequested`, `CheckInStatusSuggested`, `CheckInCompleted`, `CheckInBlocked`.
- Departure: `DeparturePrepWindowOpened`, `CheckOutStarted`, `CheckOutReadinessEvaluated`, `CheckOutReviewRequested`, `CheckOutStatusSuggested`, `CheckOutCompleted`, `CheckOutBlocked`.
- Existing generic `workflow::WorkflowEventType` may initially use `BookingTriageNeeded`, `BookingConfirmationNeeded`, `DailyUpdateNeeded`, `CheckoutCompleted`, plus `entities::AuditAction::Extension` labels until dedicated variants are justified.

### Staff tasks

`operations::boarding::task::Kind` maps to generic `operations::StaffTaskKind` while preserving Boarding source context:

- `ArrivalPrep` -> `StaffTaskKind::CheckInPrep { reservation_id }`
- `ArrivalDocumentReview` -> `StaffTaskKind::DocumentReview { pet_id }`
- `ArrivalCareReview` -> `StaffTaskKind::Feeding` / `MedicationAdministration` / `PlaygroupAssessment` with Boarding source metadata.
- `BelongingsIntake` -> generic `CheckInPrep` with Boarding task kind source.
- `DeparturePrep` -> `StaffTaskKind::CheckOutPrep { reservation_id }`
- `FinalCareReview` -> care task kind with release readiness reason.
- `BelongingsReturn` -> generic `CheckOutPrep` with Boarding task kind source.
- `ReportReview` -> `StaffTaskKind::DailyUpdateDraft { reservation_id }` or future report-specific task.
- `RoomTurnover` -> `StaffTaskKind::CleaningTurnover { reservation_id }`
- `PaymentFollowUp` -> `StaffTaskKind::CustomerFollowUp { reason: DepositNotPaid }`

### Agent specs / tools

- Agent specs:
  - `boarding-pre-arrival-checklist`: read reservation/customer/pet/care/payment/capacity; draft internal tasks and readiness packet.
  - `boarding-check-in-readiness`: summarize gaps and review gates for staff.
  - `boarding-check-out-readiness`: summarize open care/payment/report/belongings/turnover tasks.
  - `boarding-pawgress-draft`: draft report from approved staff evidence only.
  - `boarding-upsell-review`: draft internal opportunities with suppression rationale.
- Tool ports:
  - `tools::PetResortEntityStore` for entity reads.
  - `tools::ReservationSystem::check_availability` for provider availability evidence.
  - `tools::ReservationSystem::draft_reservation_update` for status-change drafts.
  - Future payment/document/media/messaging ports must require typed approvals before side effects.

## 4. Interaction contract

The following pseudo-signatures define behavior owners. They are intentionally domain-service/repository methods rather than free-floating helpers.

### Arrival readiness policy

```rust
impl operations::boarding::arrival::Policy {
    pub fn evaluate(
        &self,
        contract: &operations::boarding::Contract,
        reservation: &entities::Reservation,
        customer: &entities::Customer,
        pets: &[entities::Pet],
        capacity: &operations::boarding::capacity::Snapshot,
        care: &operations::boarding::care::Projection,
        payment: &operations::boarding::payment::Readiness,
        existing_tasks: &[operations::StaffTask],
        prior_events: &[workflow::WorkflowEvent],
    ) -> operations::boarding::arrival::Decision;
}
```

Contract:

- `arrival::Policy` owns readiness rules because check-in is a Boarding operational invariant over reservation, care, capacity, and payment evidence.
- It must not write repositories, update provider status, collect money, or send messages.
- It returns a typed decision with evidence and review gates.
- Missing/ambiguous care, payment, provider-sync, or capacity facts become `NeedsStaffReview`, `NeedsManagerApproval`, or `Blocked`; they are never silently defaulted.

### Arrival planner

```rust
impl operations::boarding::arrival::Planner {
    pub fn plan(
        &self,
        decision: &operations::boarding::arrival::Decision,
        task_policy: &operations::boarding::task::Policy,
        approval_policy: &operations::boarding::agent::ApprovalPolicy,
    ) -> operations::boarding::workflow::Plan;
}
```

Contract:

- `arrival::Planner` owns conversion from domain decision to staff tasks and workflow recommendations.
- It may create `workflow::RecommendedAction::UpdateStatus` only as a recommendation/draft with a reason.
- It should include `policy::ReviewGate` values when the approval policy says the proposed action is not fully automated.
- It should map Boarding-specific task kinds into generic `operations::StaffTask` without erasing source reason.

### Arrival repository

```rust
#[async_trait]
pub trait operations::boarding::arrival::Repository {
    async fn load_packet(
        &self,
        reservation_id: entities::ReservationId,
    ) -> operations::boarding::arrival::Result<Option<operations::boarding::arrival::Packet>>;

    async fn save_packet(
        &self,
        packet: operations::boarding::arrival::Packet,
    ) -> operations::boarding::arrival::Result<()>;

    async fn record_decision(
        &self,
        reservation_id: entities::ReservationId,
        decision: operations::boarding::arrival::Decision,
        audit: entities::AuditEvent,
    ) -> operations::boarding::arrival::Result<()>;
}
```

Contract:

- The repository persists arrival workflow state and audit links; it does not decide policy.
- Storage rows must convert into semantic packet/checklist/evidence types at the boundary.
- Recording a decision requires an audit event or workflow event reference.

### Departure readiness policy

```rust
impl operations::boarding::departure::Policy {
    pub fn evaluate(
        &self,
        contract: &operations::boarding::Contract,
        reservation: &entities::Reservation,
        pets: &[entities::Pet],
        care: &operations::boarding::care::Projection,
        payment: &operations::boarding::payment::Readiness,
        belongings: &operations::boarding::belongings::Manifest,
        report: &operations::boarding::report::Resolution,
        open_tasks: &[operations::StaffTask],
        prior_events: &[workflow::WorkflowEvent],
    ) -> operations::boarding::departure::Decision;
}
```

Contract:

- `departure::Policy` owns checkout-readiness rules because release/payment/report/belongings/turnover are Boarding operational invariants.
- It must fail closed for unresolved care evidence, payment due, report approval, belongings mismatch, and provider-sync ambiguity.
- It returns typed decision values and room-turnover/task requirements; it does not mark provider checkout complete.

### Departure planner

```rust
impl operations::boarding::departure::Planner {
    pub fn plan(
        &self,
        decision: &operations::boarding::departure::Decision,
        task_policy: &operations::boarding::task::Policy,
        upsell_policy: &operations::boarding::upsell::Policy,
        approval_policy: &operations::boarding::agent::ApprovalPolicy,
    ) -> operations::boarding::workflow::Plan;
}
```

Contract:

- Emits checkout prep, room turnover, report review, payment follow-up, incident follow-up, customer follow-up, and eligible internal upsell recommendations.
- Suppresses or review-gates upsells when the pet/customer/care context makes them unsafe or insensitive.
- Emits `UpdateStatus(CompleteCheckout)` only as an approved/reviewable recommendation.

### Departure repository

```rust
#[async_trait]
pub trait operations::boarding::departure::Repository {
    async fn load_packet(
        &self,
        reservation_id: entities::ReservationId,
    ) -> operations::boarding::departure::Result<Option<operations::boarding::departure::Packet>>;

    async fn save_packet(
        &self,
        packet: operations::boarding::departure::Packet,
    ) -> operations::boarding::departure::Result<()>;

    async fn record_decision(
        &self,
        reservation_id: entities::ReservationId,
        decision: operations::boarding::departure::Decision,
        audit: entities::AuditEvent,
    ) -> operations::boarding::departure::Result<()>;
}
```

### Belongings manifest

```rust
impl operations::boarding::belongings::Manifest {
    pub fn receive_item(
        &mut self,
        item: operations::boarding::belongings::Item,
        actor: entities::ActorRef,
        evidence: operations::boarding::readiness::EvidenceRef,
    ) -> operations::boarding::belongings::Result<()>;

    pub fn mark_returned(
        &mut self,
        item_id: operations::boarding::belongings::ItemId,
        actor: entities::ActorRef,
        evidence: operations::boarding::readiness::EvidenceRef,
    ) -> operations::boarding::belongings::Result<()>;

    pub fn reconciliation_state(&self) -> operations::boarding::belongings::Reconciliation;
}
```

Contract:

- Manifest methods own belongings state transitions because item intake/release is a domain concept, not a checklist string.
- Missing/damaged/declined items require evidence and usually a review gate before checkout completion.

### Report approval policy

```rust
impl operations::boarding::report::ApprovalPolicy {
    pub fn resolve_for_checkout(
        &self,
        report: Option<&operations::boarding::report::PawgressReport>,
        evidence: &[operations::boarding::report::SourceEvidence],
        care_flags: &[operations::boarding::care::ReviewGate],
    ) -> operations::boarding::report::Resolution;
}
```

Contract:

- Report policy owns customer-facing report state. Departure policy consumes `report::Resolution` rather than inspecting raw report text.
- Sensitive medical/behavior/incident/payment/complaint content must require manager review or suppression with a reason.

### Approval policy

```rust
impl operations::boarding::agent::ApprovalPolicy {
    pub fn classify_arrival_action(
        &self,
        action: &workflow::RecommendedAction,
        decision: &operations::boarding::arrival::Decision,
    ) -> policy::AutomationLevel;

    pub fn classify_departure_action(
        &self,
        action: &workflow::RecommendedAction,
        decision: &operations::boarding::departure::Decision,
    ) -> policy::AutomationLevel;
}
```

Contract:

- Internal summaries/task drafts may be internal-only automation.
- Status updates, payment actions, customer messages, report sends, capacity overrides, room reassignment under constraint, medical/behavior decisions, and incident closure require explicit review gates.

## 5. Review/approval contract

### Automation level

Safe/internal automation:

- Read reservation/customer/pet/care/payment/capacity projections.
- Summarize arrival/departure readiness.
- Draft arrival/departure packets and checklists.
- Draft internal staff tasks for missing evidence, open care tasks, payment follow-up, report review, room turnover, or manager review.
- Draft Pawgress Report text only from approved staff evidence and keep it in `Draft`/review state.
- Draft internal upsell recommendations with eligibility and suppression rationale.

Staff review required:

- Marking check-in checklist items complete from free-text, media, scanned documents, customer-provided info, or provider-sync evidence.
- Confirming medication/feeding/behavior/playtime readiness.
- Applying normal room assignments when there is any ambiguity in species/accommodation/capacity evidence.
- Suggesting reservation status transition to `CheckedIn` or `CheckedOut`.
- Approving non-sensitive customer-facing report/message drafts.
- Reconciling belongings exceptions that do not require manager escalation.

Manager approval required:

- Capacity override, waitlist exception, room hold release, overbook/limited-inventory assignment.
- Deposit/payment waiver, refund, forfeit, exception, unresolved payment at checkout.
- Missing/ambiguous medication, medical, behavior, incident, injury, safety, legal/regulatory, complaint, or sensitive customer communication.
- Late departure exception, extended checkout fee/waiver, or policy exception.
- Pawgress/customer message content involving medical, behavior, incident, payment, denial, complaint, or policy explanation.

Never fully automated / member-facing boundary:

- Confirming live check-in/check-out status in provider systems.
- Charging, refunding, waiving, or forfeiting money.
- Sending Pawgress Reports or customer messages.
- Approving medical/behavior/play safety, incident closure, or customer complaint resolution.
- Hiding unresolved care, payment, belongings, report, or safety exceptions from staff/customer-visible workflows.

### Review gates

Use existing `policy::ReviewGate` where it fits and add Boarding-specific gates only when the shared policy vocabulary is too vague. Existing shared gates to reuse:

- `policy::ReviewGate::ManagerApproval`
- `policy::ReviewGate::MedicalDocumentReview`
- `policy::ReviewGate::BehaviorReview`
- `policy::ReviewGate::CustomerMessageApproval`
- `policy::ReviewGate::RefundOrDepositException`

Candidate Boarding-local gates if the shared vocabulary is too coarse:

- `boarding::arrival::ReviewGate::StaffArrivalApproval`
- `boarding::arrival::ReviewGate::CareInstructionApproval`
- `boarding::arrival::ReviewGate::AccommodationAssignmentApproval`
- `boarding::arrival::ReviewGate::PaymentAtArrivalApproval`
- `boarding::departure::ReviewGate::StaffReleaseApproval`
- `boarding::departure::ReviewGate::FinalCareReleaseApproval`
- `boarding::departure::ReviewGate::BelongingsExceptionApproval`
- `boarding::departure::ReviewGate::ReportSendApproval`
- `boarding::departure::ReviewGate::PaymentAtDepartureApproval`
- `boarding::departure::ReviewGate::LateDepartureExceptionApproval`

If shared `policy::ReviewGate` is expanded later, Boarding-specific gates should convert explicitly rather than leak string labels.

### Audit trail

Every arrival/departure decision should record:

- actor (`entities::ActorRef`) and source (`workflow::WorkflowEventId`, staff action, provider external ID);
- reservation ID, location ID, customer ID, and pet IDs involved;
- decision type and typed reasons/blocks;
- evidence refs used to satisfy checklist items;
- required reviews and who approved them;
- status update draft ID / provider update reference if executed after approval;
- report/message/payment exception boundaries;
- suppressed recommendations and reasons.

Audit events should use existing `entities::AuditAction::{PolicyDecisionRecorded, ReservationStatusSuggested, ReservationStatusChanged, WorkflowEventRecorded, Extension(...)}` until a dedicated Boarding audit enum is justified.

## 6. Test contracts

Future implementation cards should add named semantic tests like these:

### Arrival/check-in tests

- `boarding_arrival_policy_blocks_checkin_for_cancelled_or_rejected_reservation`
  - A non-arrival-eligible status returns `arrival::Decision::Blocked`, not a status update.
- `boarding_arrival_policy_requires_confirmed_or_staff_reviewed_offered_reservation`
  - Offered/missing-info reservations cannot silently become checked in.
- `boarding_arrival_packet_requires_all_pet_entries_to_match_reservation`
  - Packet construction rejects pet readiness entries not attached to the reservation.
- `boarding_arrival_policy_blocks_accommodation_species_mismatch`
  - Cat condo/dog suite mismatches produce typed accommodation block reasons.
- `boarding_arrival_policy_routes_missing_room_assignment_to_staff_or_manager_review`
  - No valid room assignment never becomes automatic check-in.
- `boarding_arrival_policy_flags_missing_feeding_or_medication_instruction_before_checkin_complete`
  - Missing care instructions create review/task outputs.
- `boarding_arrival_policy_requires_deposit_review_when_payment_due_at_checkin_is_unpaid`
  - Deposit/payment readiness blocks or review-gates arrival.
- `boarding_arrival_planner_creates_internal_tasks_for_document_care_payment_and_belongings_gaps`
  - Planner emits staff tasks, not provider side effects.
- `boarding_arrival_status_update_is_recommended_action_until_staff_approved`
  - `CheckedIn` transition appears as reviewable recommendation/draft only.
- `boarding_arrival_agent_cannot_approve_medical_behavior_or_payment_exceptions`
  - Approval policy classifies these actions as human review/manager required.

### Departure/check-out tests

- `boarding_departure_policy_blocks_checkout_when_reservation_not_checked_in_or_active`
  - Checkout cannot complete from requested/offered/cancelled states.
- `boarding_departure_policy_requires_open_care_tasks_to_be_completed_or_review_resolved`
  - Open medication/feeding/incident/final care tasks block release.
- `boarding_departure_policy_requires_payment_due_at_checkout_to_be_paid_or_manager_approved`
  - Payment exception is explicit and review-gated.
- `boarding_departure_policy_requires_belongings_manifest_reconciliation`
  - Missing/damaged/unreturned items block or review-gate checkout.
- `boarding_departure_policy_requires_pawgress_report_resolution_when_report_is_included`
  - Included report must be approved/sent/suppressed with reason before checkout completion.
- `boarding_departure_policy_routes_sensitive_report_content_to_manager_review`
  - Medical/behavior/incident/payment/complaint content is not staff-only auto approval.
- `boarding_departure_planner_creates_room_turnover_task_before_returning_room_to_inventory`
  - Checkout emits cleaning turnover before capacity release.
- `boarding_departure_planner_suppresses_upsell_when_care_or_customer_context_is_unsafe`
  - Exit bath/grooming/training recommendations are not created when incompatible.
- `boarding_departure_status_update_is_recommended_action_until_staff_approved`
  - `CheckedOut` transition remains a draft/recommended action.
- `boarding_departure_agent_never_sends_pawgress_or_customer_message_without_review_gate_clearance`
  - Customer-facing send remains blocked without approval.

### Shared readiness/audit/storage tests

- `boarding_belongings_manifest_cannot_complete_with_missing_required_item_without_review`
- `boarding_readiness_evidence_ref_preserves_source_type_and_identifier`
- `boarding_checkin_checkout_decisions_record_policy_context_reviews_and_actor_in_audit_event`
- `boarding_task_mapping_preserves_boarding_task_kind_when_projected_to_operations_staff_task`
- `boarding_arrival_departure_packets_roundtrip_through_storage_without_raw_status_strings`
- `boarding_provider_sync_failure_fails_closed_into_staff_review`

## 7. Integration notes for serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add child modules under `operations::boarding`: `arrival`, `departure`, `readiness`, `belongings`, and possibly `task`/`report` refinements.
  - Keep `boarding::Contract` stable; add fields only if serialized storage migration is deliberate.
- `domain/src/entities.rs`
  - Reuse `ReservationStatus::{CheckedIn, Active, CheckedOut}`, `AuditEvent`, `AuditAction`, `ActorRef`, IDs, and hard stops. Avoid creating Boarding-specific duplicates of core entity IDs.
- `domain/src/workflow.rs`
  - Reuse `RecommendedAction`, `PolicyContext`, `ReviewReason`, `AllowedAction`, and `WorkflowEvent`. Add event/allowed-action variants only if generic vocabulary is insufficient.
- `domain/src/policy.rs`
  - Add or refine review gates/automation levels if Boarding-specific arrival/departure gates cannot be expressed with existing policy values.
- `domain/src/tools.rs`
  - Reuse `ReservationSystem::draft_reservation_update`. Future payment/document/message tools should require typed approval inputs.
- `domain/src/storage/...` or `domain/src/storage/operations...`
  - Add storage records/codecs for arrival/departure packets only after domain types are stable.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Preserve existing Boarding contract tests.
- New tests, likely `domain/tests/boarding_checkin_checkout_flows.rs`, for semantic contract tests listed above.
- `docs/domain/petsuites/boarding/service-domain-map.md`
  - Link back to this implication if docs consolidation happens later.

### Migration/refactor risks

- `operations::boarding::Contract` is already serialized through service-contract storage. Adding non-default required fields will break existing roundtrips unless migration/defaulting is explicit.
- Generic `operations::StaffTaskKind::{CheckInPrep, CheckOutPrep}` is too coarse for Boarding semantics. Preserve source context rather than flattening all arrival/departure obligations into generic task titles.
- `entities::ReservationStatus` is cross-service. Do not fork a separate Boarding status lifecycle; use Boarding `stay::Phase`/readiness as projection plus recommended status transitions.
- Payment boundaries are easy to blur. Boarding should evaluate `payment::DepositStatus` against policy, but transaction changes belong to payment/tool ports with approval.
- Pawgress Report text is customer-facing. Treat report drafts, approvals, sends, and suppression reasons as typed state transitions.
- Multi-pet reservations can make readiness partially complete. Avoid reservation-level booleans that hide per-pet care or belongings exceptions.
- Provider sync can be stale or unavailable. Policy should fail closed into staff review instead of assuming check-in/check-out success.
- Room turnover affects capacity. Checkout completion and capacity release are related but not identical; room cleaning/turnover should be explicit.

### Dependencies on other implications

- Capacity/accommodation segmentation: arrival needs valid room assignment and species/accommodation compatibility.
- Care/playtime/handoff implication: arrival/departure consume care readiness, medication/feeding/behavior review, and handoff tasks.
- Deposit/cancellation/payment implication: arrival/departure need typed payment/deposit readiness and exception gates.
- Pawgress Report/customer communication implication: checkout consumes report resolution and approval state.
- Upsell/revenue implication: checkout may recommend exit bath, grooming, training, or rebooking internally with approval/suppression rationale.
- Agent approval-policy implication: both flows require deterministic approval classification before tool execution.

### Suggested implementation sequence

1. Add pure domain value objects/enums for `readiness`, `arrival`, `departure`, and `belongings` without storage changes.
2. Add `arrival::Policy` and `departure::Policy` tests using in-memory fixtures and existing entity/payment/workflow types.
3. Add planners that convert decisions into `operations::StaffTask` and `workflow::RecommendedAction` values.
4. Add approval-policy tests proving agents can draft but cannot execute check-in/check-out/payment/message/report side effects.
5. Only then add storage records/codecs and provider/tool adapters, with explicit migration tests for serialized service-contract compatibility.
