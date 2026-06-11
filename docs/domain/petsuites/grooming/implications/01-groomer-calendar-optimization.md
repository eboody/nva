# Grooming operational implication 01: Groomer calendar optimization

Purpose: model the grooming-specific calendar optimization workflow for PetSuites as a domain contract for later Rust implementation. This is a modeling artifact, not an automation permission. The optimizer may rank windows, expose conflicts, prepare draft holds, and route review tasks; it must not silently book, reschedule, double-book, charge, discount, or send customer/member-facing messages.

Source context:

- `docs/domain/petsuites/grooming/service-domain-map.md`
- Existing Rust surface in `domain/src/operations.rs`, especially `operations::grooming::{Contract, CalendarPolicy, BreedCoatTimeEstimate, AppointmentMinutes, NoShowPolicy, RebookingCadence, ReminderRule, HistoryRequirement}`.
- Existing tests in `domain/tests/petsuites_core_service_contracts.rs` and `domain/tests/domain_quality_patterns.rs` that already protect the standard grooming contract, service offering variants, agent specs, and grooming capacity/revenue concepts.

Assumptions:

- PetSuites grooming calendars are location-specific and may use provider calendars such as Gingr plus local staff schedule knowledge.
- A groomer calendar optimization recommendation is safe when it is a ranked, explainable proposal. Calendar writes are limited to draft holds unless a typed approval token authorizes a provider-side booking/reschedule.
- Groomer qualification, preferred/required groomer, buffers, service duration, care-sensitive coat/product issues, and no-show/deposit history are all deterministic policy inputs, not LLM-only judgment.
- Unknown business details should become explicit review reasons rather than permissive defaults.

## 1. Operational story

### Trigger

A calendar optimization run begins from one of these domain events or staff/agent requests:

1. A new grooming `AppointmentRequest` needs viable appointment windows.
2. Staff asks for same-day or next-day grooming slot fill opportunities.
3. Boarding/daycare checkout context suggests an exit bath or grooming add-on, but only as a draft opportunity.
4. Rebooking policy marks a pet due or overdue and wants ranked follow-up windows.
5. A groomer calendar changes: cancellation, no-show, staff absence, extended service, blackout/lunch block, or manager override.
6. A manager requests a bottleneck summary for groomer utilization, unfilled blocks, or conflicts.

### Actors

- Customer/member: owns preferences, communication consent, and final customer-facing commitment.
- Pet: contributes species/breed/coat facts through pet/care profile values.
- Groomer/staff: owns professional judgment for style interpretation, care-sensitive handling, and service completion notes.
- Manager: owns override decisions, double-booking/buffer exceptions, deposit/penalty waivers, and staffing tradeoffs.
- AI/calendar optimizer: ranks windows, explains conflicts, drafts holds/tasks/messages, and surfaces evidence.
- Provider calendar/tool boundary: reads availability and may create a draft hold or confirmed booking only after deterministic policy plus approval allows it.

### Inputs

- `operations::grooming::Contract` for the location, including calendar policy, standard estimates, no-show/rebooking/reminder/history rules.
- `operations::grooming::AppointmentRequest` with location, customer, pet, offering/service plan, source context, optional reservation link, optional preferred/required groomer, and requested window.
- Groomer availability and existing `CalendarBlock`s from `operations::grooming::CalendarRepository`.
- Staff qualification snapshots, modeled as references to `staff::Id`, `StaffRole::Groomer`, and grooming skill/permission values rather than raw names.
- Pet profile facts: species/breed/size/coat condition and care-sensitive references owned by `pet`/`care` modules.
- Service history, style notes, prior duration, no-show/late-cancel history, and rebooking cadence from `operations::grooming::HistoryRepository`.
- Customer communication consent/preferences and reservation checkout context from neighboring customer/reservation contexts.
- Existing holds and draft recommendations, so the optimizer does not duplicate pending work.

### Decisions

The workflow makes these decisions in order:

1. Estimate service duration and review requirements.
   - Positive appointment minutes are mandatory.
   - Missing breed/coat/size, matted coat, sensitive skin, medical product need, or style ambiguity produces staff/groomer review.
2. Determine assignment constraints.
   - `CalendarPolicy::GroomerSpecific` means the optimizer must honor a named/qualified groomer or explicitly explain why no candidate exists.
   - `AnyQualifiedGroomer` may rank across eligible groomers.
   - `FirstAvailableWithManagerOverride` may propose an override candidate but cannot apply the override.
3. Build feasible calendar candidates.
   - Candidates must include duration plus required buffers.
   - Candidates must not overlap appointments, holds, staff unavailable blocks, blackout/lunch blocks, or manager-protected blocks.
   - If the request is linked to boarding/daycare checkout, the optimizer may prefer windows that fit pickup/check-out timing, but grooming does not mutate the reservation.
4. Rank candidates.
   - Ranking can consider requested window fit, groomer preference, service complexity, utilization smoothing, checkout convenience, rebooking overdue status, and cancellation/no-show risk.
   - Ranking must preserve explanation and should not hide hard constraint failures.
5. Choose output boundary.
   - Safe outputs: ranked candidates, conflict explanation, draft hold proposal, staff task, manager review task, draft customer message.
   - Unsafe without approval: confirmed booking, reschedule, cancellation, deposit/charge/waiver, member-facing send.

### Outputs

- `operations::grooming::CalendarOptimization` containing feasible `ScheduleCandidate`s, rejected candidates with typed conflict reasons, and an overall `OptimizationOutcome`.
- Optional `DraftCalendarHold` for the best candidate when the provider/tool supports a non-committing hold and policy allows it.
- `workflow::RecommendedAction` or staff task for review-required cases.
- Draft customer/member message for confirmation or rebooking follow-up, gated by communication consent and approval.
- Audit event describing inputs snapshot, policy version, candidate ranking rationale, review boundary, and any tool command proposed/executed.

### Success state

The workflow succeeds when it produces an explainable scheduling decision without violating calendar, care, payment, communication, or approval boundaries. A fully successful automated run may create only draft/non-committing artifacts unless a typed approval token is present. A staff-approved run may execute a provider calendar booking/reschedule through the tool boundary and persist an audit trail.

### Failure and exception states

- `NoViableWindow`: every candidate conflicts with groomer availability, buffers, requested window, or service duration.
- `MissingCalendarSnapshot`: provider calendar or staff schedule data is stale/unavailable.
- `UnknownDurationRequiresReview`: estimate cannot be made safely because service, breed, coat, or history inputs are missing.
- `CareSensitiveReviewRequired`: matted coat, sensitive skin, medical-product need, or behavior/handling concern requires groomer/staff review.
- `UnqualifiedGroomer`: preferred or assigned groomer lacks the required qualification or policy permission.
- `ManagerOverrideRequired`: candidate needs double-booking, buffer compression, blackout override, deposit/penalty waiver, or protected-block release.
- `NoShowDepositReviewRequired`: customer/pet history requires deposit, hold restriction, or manager review before booking.
- `CommunicationConsentMissing`: draft message may exist, but no customer-facing send is allowed.
- `ProviderWriteRejected`: external calendar refused draft hold or booking command; domain preserves failed command/audit without pretending the appointment is scheduled.
- `DuplicatePendingHold`: a pending hold/recommendation already covers the same pet/customer/window and should be consolidated.

## 2. Domain types to add or refine

Prefer the semantic surface under `operations::grooming`; neighboring concepts are referenced through typed IDs or ports, not copied into grooming.

### Calendar request and plan types

- `operations::grooming::AppointmentRequest`
  - Required: `location::Id` or `entities::LocationId`, `entities::CustomerId`, `entities::PetId`, `ServiceOffering`, `RequestSource`.
  - Optional typed fields: `RequestedWindow`, `ReservationLink`, `PreferredGroomer`, `StyleNote`, `ProductNeed`, `FirstTimeOfferIntent`.
  - Invariant: cannot be schedulable without location/customer/pet/offering/source; provider IDs are parsed at boundaries.
- `operations::grooming::ServicePlan`
  - Accepted bundle of offering, options/product needs, duration estimate, assignment constraints, price/deposit quote refs, and review state.
  - Invariant: service plan can reference money/payment quotes but cannot capture payment.
- `operations::grooming::ScheduleWindow`
  - Typed time interval with start < end, timezone/location context, and helper-free overlap semantics.
  - Invariant: no naked `(DateTime, DateTime)` pairs in scheduling policy contracts.
- `operations::grooming::CalendarBlock`
  - Variants: `Appointment`, `DraftHold`, `Buffer`, `Blackout`, `Lunch`, `StaffUnavailable`, `ManagerProtected`, `ProviderUnknown`.
  - Invariant: every block has a `ScheduleWindow`; appointment/hold blocks carry appointment/hold IDs when known.
- `operations::grooming::BufferMinutes`
  - Non-negative or positive-by-policy newtype. The location `Contract` or `CalendarPolicy` should say whether zero buffer is legal.
- `operations::grooming::DraftHoldId`
  - Non-empty ID for provider/domain draft holds.
- `operations::grooming::AppointmentId`
  - Non-empty domain/provider appointment ID after persistence.

### Assignment, qualification, and ranking types

- `operations::grooming::GroomerAssignment`
  - Variants: `AnyQualified`, `Preferred(staff::Id)`, `Required(staff::Id)`, `ManagerOverride(staff::Id)`.
  - Invariant: `Required` cannot silently degrade to `AnyQualified`; degradation is a review decision.
- `operations::grooming::GroomerQualification`
  - Examples: `Bath`, `Haircut`, `DoodleOrPoodleMix`, `CatGrooming`, `MedicatedProductReview`, `ManagerOverrideAllowed`.
  - Invariant: qualifications are staff/location facts, not strings in calendar blocks.
- `operations::grooming::AssignmentDecision`
  - Variants: `Eligible`, `Rejected { reason: AssignmentRejection }`, `ManagerReviewRequired { reason }`.
- `operations::grooming::ScheduleCandidate`
  - Fields: groomer, window, service plan, duration estimate, buffer, rank, rationale, approval boundary.
  - Invariant: a candidate is feasible only if hard calendar/qualification constraints pass; otherwise represent it as a rejected candidate/conflict.
- `operations::grooming::CandidateRank`
  - Positive ordinal or bounded score with explanation. Prefer rank plus rationale over opaque LLM score.
- `operations::grooming::OptimizationRationale`
  - Structured reasons: `FitsRequestedWindow`, `PreferredGroomer`, `CheckoutAligned`, `FillsCancellationGap`, `BalancesUtilization`, `RebookingOverdue`, `MinimizesIdleFragment`.

### Estimation and review types

- Refine `operations::grooming::DurationEstimate`
  - Contains `AppointmentMinutes`, `EstimateBasis`, `ConfidenceBasisPoints`, and `ReviewRequirement`.
  - Invariant: low confidence or sensitive basis cannot produce auto-executable booking.
- Refine `operations::grooming::EstimateBasis`
  - Variants: `BreedCoatPolicy`, `GroomerHistory`, `ProviderDefault`, `ManualStaffOverride`, `AiSuggestedPendingReview`.
- Add/extend `operations::grooming::ReviewRequirement`
  - Variants: `None`, `StaffReview`, `GroomerReview`, `ManagerReview`, `CareOrMedicalReview`.
- Extend `BreedCategory`/`CoatCondition` safely over time.
  - Add unknown/review variants instead of defaulting unknown coat to maintained.

### Conflict, outcome, and audit types

- `operations::grooming::CalendarConflict`
  - Variants: `OverlapsAppointment`, `OverlapsDraftHold`, `MissingBuffer`, `GroomerUnavailable`, `Blackout`, `LunchBlock`, `UnqualifiedGroomer`, `OutsideRequestedWindow`, `DurationUnknown`, `ProviderDataStale`, `ManagerProtectedBlock`.
  - Invariant: conflict carries typed block/window/groomer context when available.
- `operations::grooming::OptimizationOutcome`
  - Variants: `RankedCandidates`, `DraftHoldProposed`, `ReviewRequired`, `NoViableWindow`, `ProviderUnavailable`.
- `operations::grooming::CalendarOptimization`
  - Aggregate result with request, policy snapshot ID/version, feasible candidates, rejected conflicts, recommendation, automation decision, audit facts.
- `operations::grooming::CalendarOptimizationEvent`
  - Workflow event variants: `Requested`, `CandidatesRanked`, `DraftHoldProposed`, `ReviewTaskCreated`, `Approved`, `ProviderCommandIssued`, `ProviderCommandRejected`, `ConfirmedBookingRecorded`.
- `operations::grooming::ApprovalBoundary`
  - Domain-facing gate: `DraftOnly`, `StaffApprovalRequired`, `GroomerApprovalRequired`, `ManagerApprovalRequired`, `MemberFacingSendRequiresApproval`, `ApprovedForToolExecution`, `Disallowed`.

## 3. Relationship map between types

### Entities and value objects

- `AppointmentRequest` references `entities::CustomerId`, `entities::PetId`, `entities::LocationId`, optional `reservation::Id`, and optional `staff::Id` through assignment values.
- `ServicePlan` owns grooming offering/options and references payment quote/deposit decisions by typed IDs or value objects from money/payment modules.
- `DurationEstimate`, `AppointmentMinutes`, `BufferMinutes`, `ScheduleWindow`, `CalendarBlock`, and `CalendarConflict` are value objects owned by `operations::grooming`.
- `AppointmentId` and `DraftHoldId` are entity IDs for persisted appointments/holds.
- `ServiceHistoryEntry` feeds duration and rebooking decisions but stays separate from care/medical records.

### Policies

- `EstimationPolicy` owns duration/review computation.
- `SchedulingPolicy` owns feasible-window generation and conflict detection.
- `CalendarOptimizationPolicy` owns candidate ranking and optimization outcome selection.
- `NoShowPolicyEngine` owns deposit/hold/review implications from customer/pet history.
- `AutomationPolicy` owns final action boundary, using risk, confidence, consent, and review requirements.
- `ReminderPolicy` owns draft reminder/follow-up plans after scheduling decisions.

### Repositories and stores

- `operations::grooming::ContractRepository` loads location policy snapshots.
- `operations::grooming::CalendarRepository` reads availability/blocks and writes draft holds only through explicit contract methods.
- `operations::grooming::HistoryRepository` loads service history and prior durations/style notes.
- `operations::grooming::AppointmentRepository` persists domain appointment/optimization state after provider boundary conversion.
- Storage modules keep provider payloads as records, then promote them into domain types. Provider-specific strings do not leak into policy code.

### Workflow events and staff tasks

- `CalendarOptimizationEvent` records domain workflow transitions and provider/audit facts.
- Staff task kinds should be explicit when existing generic kinds are insufficient:
  - `GroomingCalendarReview`
  - `GroomingDurationReview`
  - `GroomingManagerOverrideReview`
  - `GroomingCustomerFollowUp`
- Existing staff task surfaces may be reused only when the call site remains truthful, e.g. `CustomerFollowUp` for a draft confirmation review.

### Agent specs and tools

- `agents::grooming-rebooking` can be extended or paired with a calendar optimizer spec that has read-only grooming history/availability tools plus draft-message/task permissions.
- The agent input should be a typed prompt packet: request, calendar snapshot, history summary, constraints, approval boundary.
- Agent output should be typed: ranked windows, rationale, conflicts, risk flags, review reasons, draft message. It is not an execution command.
- Calendar/POS/messaging tools are boundary ports. Tool execution requires an `ApprovedForToolExecution` decision and writes an audit event.

## 4. Interaction contract

Rust-like pseudo-signatures below are intentionally precise about ownership. They are not final code.

```rust
impl operations::grooming::AppointmentRequest {
    pub fn builder() -> appointment_request::Builder<MissingLocation, MissingCustomer, MissingPet, MissingOffering, MissingSource>;
    pub fn requested_window(&self) -> Option<&operations::grooming::ScheduleWindow>;
    pub fn assignment(&self) -> operations::grooming::GroomerAssignment;
}

impl operations::grooming::ScheduleWindow {
    pub fn try_new(
        starts_at: time::OffsetDateTime,
        ends_at: time::OffsetDateTime,
        location_tz: location::TimeZone,
    ) -> operations::grooming::schedule_window::Result<Self>;

    pub fn overlaps(&self, other: &operations::grooming::ScheduleWindow) -> bool;
    pub fn can_contain(&self, duration: operations::grooming::AppointmentMinutes, buffer: operations::grooming::BufferMinutes) -> bool;
}

trait operations::grooming::ContractRepository {
    fn load_for_location(
        &self,
        location: entities::LocationId,
    ) -> operations::grooming::Result<operations::grooming::Contract>;
}

trait operations::grooming::CalendarRepository {
    fn load_snapshot(
        &self,
        location: entities::LocationId,
        horizon: operations::grooming::ScheduleWindow,
    ) -> operations::grooming::Result<operations::grooming::CalendarSnapshot>;

    fn create_draft_hold(
        &self,
        proposal: operations::grooming::DraftHoldProposal,
        approval: policy::ApprovalToken,
    ) -> operations::grooming::Result<operations::grooming::DraftHoldId>;
}

trait operations::grooming::HistoryRepository {
    fn service_history_for_pet(
        &self,
        pet: entities::PetId,
    ) -> operations::grooming::Result<Vec<operations::grooming::ServiceHistoryEntry>>;
}

impl operations::grooming::EstimationPolicy {
    pub fn estimate(
        &self,
        request: &operations::grooming::AppointmentRequest,
        pet_profile: &pet::Profile,
        care_summary: Option<&care::GroomingRelevantSummary>,
        history: &[operations::grooming::ServiceHistoryEntry],
        contract: &operations::grooming::Contract,
    ) -> operations::grooming::DurationEstimate;
}

impl operations::grooming::SchedulingPolicy {
    pub fn feasible_candidates(
        &self,
        request: &operations::grooming::AppointmentRequest,
        service_plan: &operations::grooming::ServicePlan,
        calendar: &operations::grooming::CalendarSnapshot,
        contract: &operations::grooming::Contract,
    ) -> operations::grooming::ScheduleCandidateSet;
}

impl operations::grooming::CalendarOptimizationPolicy {
    pub fn optimize(
        &self,
        request: operations::grooming::AppointmentRequest,
        candidates: operations::grooming::ScheduleCandidateSet,
        no_show: operations::grooming::NoShowDecision,
        consent: customer::CommunicationConsent,
        now: time::OffsetDateTime,
    ) -> operations::grooming::CalendarOptimization;
}

impl operations::grooming::AutomationPolicy {
    pub fn decide_calendar_action(
        &self,
        optimization: &operations::grooming::CalendarOptimization,
        actor: workflow::Actor,
    ) -> operations::grooming::ApprovalBoundary;
}
```

Behavior ownership rules:

- `ScheduleWindow` owns interval validation/overlap semantics.
- `EstimationPolicy` owns duration and review requirement, not `SchedulingPolicy` and not the agent.
- `SchedulingPolicy` owns hard calendar feasibility and conflict explanation.
- `CalendarOptimizationPolicy` owns ranking and outcome selection after feasibility is known.
- `AutomationPolicy` owns execution boundary. Agent output cannot bypass it.
- `CalendarRepository` owns provider calendar reads/draft-hold writes as a port; it does not decide whether a booking is allowed.
- Money/payment, customer consent, care/medical, reservation mutation, and messaging delivery remain in their neighboring modules/tools.

## 5. Review and approval contract

### Automation level

Default automation level: recommendation/draft-only.

Safe without human approval:

- Read location grooming contract, calendar snapshots, and service history.
- Produce ranked schedule candidates and conflict explanations.
- Detect no viable windows, missing data, stale provider reads, and review requirements.
- Draft staff tasks and customer-message text, without sending.
- Propose a draft hold only when the provider/tool semantics guarantee it is non-committing and policy allows draft holds.

Requires staff or groomer approval:

- Low-confidence estimates.
- Missing/unknown breed, coat, size, service plan, or style instructions.
- Matted coat, sensitive skin, medical-product concern, behavior/handling reference, or care-sensitive product choice.
- Interpreting style history into a new service plan.
- Converting a ranked candidate into a confirmed booking when local policy requires staff review.

Requires manager approval:

- Double-booking, compressed buffers, blackout/protected-block override, required-groomer override, staff absence override.
- Rebooking after repeated no-shows/late cancellations when a deposit, restriction, or waiver is involved.
- Deposit/discount/penalty/first-time-offer exceptions.
- Any provider write after conflicting snapshots or stale data.

Member-facing/customer boundary:

- No confirmation, reminder, rebooking offer, prep instruction, price/deposit request, apology, or cancellation/reschedule notice is sent without communication consent and approval.
- Calendar optimization can draft a message that references a proposed window. It cannot represent the appointment as booked until provider booking state confirms it.

### Audit trail

Every optimization run should capture:

- Request ID/correlation ID and actor.
- Location contract snapshot/version.
- Calendar snapshot timestamp/provider watermark.
- Inputs used for duration estimate and no-show policy, with sensitive care facts referenced rather than copied.
- Candidate list, rejection conflicts, ranking rationale, and selected recommendation.
- Automation decision and review gate.
- Staff/manager approval token when present.
- Provider commands attempted and provider results.
- Customer/member-facing drafts and send approvals, if any.

Audit entries must avoid raw PII in generic logs; durable domain records should hold typed IDs and reference sensitive notes through appropriate modules.

## 6. Test contracts

Domain tests should read like semantic glossary entries:

1. `groomer_calendar_optimizer_ranks_only_non_overlapping_candidates_with_required_buffers`
   - Given existing appointment/hold/blackout blocks, candidates that overlap or omit required buffer are rejected with `CalendarConflict`.
2. `groomer_specific_calendar_policy_rejects_any_qualified_substitution_without_review`
   - A request requiring a named groomer cannot silently fall back to another groomer.
3. `first_available_with_manager_override_marks_override_candidate_manager_review_required`
   - Override candidates may be proposed, but their approval boundary is manager review.
4. `matted_or_sensitive_coat_duration_estimate_blocks_auto_calendar_execution`
   - Estimation may return minutes, but review requirement prevents automatic booking.
5. `unknown_breed_or_coat_produces_unknown_duration_review_reason_not_default_short_slot`
   - Missing facts produce explicit review rather than a permissive default.
6. `no_show_deposit_policy_prevents_confirmed_booking_without_deposit_or_manager_review`
   - Repeated no-show/late-cancel history changes approval boundary.
7. `boarding_checkout_exit_bath_candidate_links_reservation_without_mutating_it`
   - Checkout-aligned exit bath remains a grooming proposal linked to reservation context.
8. `draft_hold_creation_requires_non_committing_provider_semantics_or_approval_token`
   - Calendar repository does not create provider-visible commitment without approval.
9. `calendar_optimizer_explains_no_viable_window_with_typed_conflicts`
   - No viable window returns conflict inventory rather than a generic failure string.
10. `candidate_ranking_preserves_rationale_for_utilization_and_customer_fit`
    - Ranking includes structured reasons such as cancellation gap fill, requested-window fit, and checkout alignment.
11. `customer_message_draft_for_proposed_window_requires_consent_and_send_approval`
    - Drafts can be produced, but member-facing send remains gated.
12. `provider_write_rejection_records_audit_event_without_marking_appointment_scheduled`
    - External failure does not corrupt domain state.
13. `duplicate_pending_hold_is_consolidated_not_double_booked`
    - Existing draft hold/recommendation prevents duplicate calendar pressure for the same pet/window.
14. `calendar_snapshot_staleness_routes_to_review_before_tool_execution`
    - Stale provider reads cannot drive booking commands.
15. `agent_calendar_recommendation_never_executes_booking_without_automation_policy_approval`
    - LLM/agent output is input to deterministic policy, not execution permission.

Storage/boundary tests:

1. `grooming_calendar_block_record_promotes_provider_blocks_to_semantic_calendar_blocks`
2. `grooming_calendar_block_record_rejects_zero_or_inverted_windows`
3. `grooming_draft_hold_record_distinguishes_hold_from_confirmed_booking`
4. `grooming_provider_calendar_write_response_maps_rejected_status_to_provider_write_rejected_event`
5. `grooming_schedule_candidate_record_roundtrips_rank_rationale_and_approval_boundary`

Workflow/agent tests:

1. `grooming_calendar_optimizer_agent_returns_ranked_candidates_conflicts_and_review_reasons`
2. `grooming_calendar_optimizer_agent_drafts_task_not_provider_command_for_manager_override`
3. `grooming_calendar_optimizer_agent_redacts_sensitive_care_details_in_prompt_and_audit_packet`

## 7. Integration notes for later serialized Rust code card

Likely files/modules touched:

- `domain/src/operations.rs`
  - Short term: extend `operations::grooming` with calendar/request/candidate/conflict/optimization types and policies.
  - Later: consider splitting `operations.rs` into semantic module files once grooming/boarding/daycare/training/retail grow beyond simple contracts.
- `domain/src/agents.rs`
  - Add or refine a calendar-optimizer-capable agent spec. Existing `grooming-rebooking` already forbids booking slots and sending messages without approval; preserve that boundary.
- `domain/src/workflow.rs`
  - Add staff task/action vocabulary only where existing kinds cannot truthfully represent grooming calendar review.
- `domain/src/tools.rs`
  - Add provider calendar tool command/result shapes for read snapshot, draft hold, approved booking/reschedule, and provider rejection if the tool layer exists here.
- `domain/src/entities.rs`, `domain/src/pet.rs`, `domain/src/care.rs`, `domain/src/customer.rs`, `domain/src/reservation/mod.rs`, `domain/src/money/mod.rs`, `domain/src/payment/mod.rs`
  - Prefer references to existing typed IDs/summaries. Do not move ownership of pet/care/customer/reservation/payment facts into grooming.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add focused calendar optimization tests adjacent to current grooming contract tests.
- `domain/tests/domain_quality_patterns.rs`
  - Add semantic quality tests for agent boundary, staff task/action shape, and service offering interactions if needed.
- Storage tests/modules, if present in later code card, should add calendar block/draft hold/optimization record codecs while keeping provider payloads at the boundary.

Migration/refactor risks:

- Duplicate cadence types already exist: `operations::CadenceWeeks` and `operations::grooming::CadenceWeeks`. Do not add a third duration/cadence primitive; choose the owner deliberately.
- Current `operations::grooming::CadenceWeeks` enforces only non-zero weeks. Ordinary rebooking policy should enforce the 2-8 week band through `RebookingCadence` or a named policy, not by silently changing generic scalar semantics if existing serialized records rely on it.
- Existing `GroomingService` is top-level under `operations`; moving to `operations::grooming::ServiceOffering` needs storage compatibility or conversion tests.
- Calendar `DateTime`/timezone representation should be chosen once. Avoid naive timestamps and avoid raw tuple windows.
- Staff/groomer IDs may not yet have a dedicated domain module. Use the project’s existing staff identity surface if present; otherwise introduce the smallest semantic `staff::Id`/`StaffRole::Groomer` surface needed.
- Provider calendars may distinguish holds, pending appointments, requests, and confirmed bookings differently. Model provider states at storage/tool boundaries and promote only truthful domain states.
- Audit and prompt packets must not copy sensitive care/medical notes into generic grooming logs.

Dependencies on other implications/workflows:

- Duration estimation and service history implications should supply `DurationEstimate`, `EstimateBasis`, and `ServiceHistoryEntry` semantics.
- No-show/cancellation implications should supply `NoShowDecision`, deposit/review boundary, and late-cancel vocabulary.
- Reminder/rebooking implications should consume optimization outputs for draft customer messages but must own send timing/consent.
- Boarding/daycare cross-sell implications may supply checkout-aligned opportunity context; grooming optimization must not mutate reservation/add-on/payment state.
- Approval/audit infrastructure should supply `policy::ApprovalToken`, review gates, and durable event recording before provider writes become executable.

Implementation entry rule:

Write failing semantic tests first for feasibility, ranking rationale, approval boundaries, and provider-write audit behavior. Then implement the smallest `operations::grooming` types/policies needed to satisfy those tests. Do not implement the optimizer as a free-floating helper over raw provider payloads; preserve meaning in domain types, repository ports, policy owners, and call sites.
