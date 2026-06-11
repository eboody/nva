# Grooming implication 02: breed/coat/time-estimate prediction

Purpose: define the grooming-domain contract for predicting appointment time from breed, coat, service, pet profile, and history. This is a modeling artifact for later Rust/domain work. It intentionally keeps provider payloads, LLM prompts, storage rows, customer identity, care/medical facts, reservation mutation, and message delivery outside `operations::grooming` until they are promoted through typed ports and approval gates.

Source context:

- Parent service map: `docs/domain/petsuites/grooming/service-domain-map.md`.
- Current code surface: `domain/src/operations.rs` already exposes `operations::grooming::{Contract, CalendarPolicy, BreedCategory, CoatCondition, BreedCoatTimeEstimate, AppointmentMinutes, NoShowPolicy, RebookingCadence, ReminderRule, HistoryRequirement}` plus cross-service `operations::{GroomingService, GroomingCadence, ServiceOffering}`.
- Current tests: `domain/tests/petsuites_core_service_contracts.rs` verifies the standard PetSuites grooming contract encodes a doodle/matted 180-minute estimate and rejects zero-minute estimates.

Assumptions:

- A breed/coat/time estimate is an operational scheduling input, not a customer-facing medical/care judgment.
- Missing or low-confidence breed/coat data should not block modeling; it should produce a typed review requirement and a safe draft plan.
- Staff can override an estimate, but the override must carry a typed actor, reason, and audit trail.
- AI/statistical agents may suggest estimates and rationales; deterministic policies decide review level and execution boundary.

## 1. Operational story

### Trigger

A time estimate is needed when any of these events happens:

1. A customer, front desk agent, groomer, or AI-assisted intake flow drafts a grooming appointment request.
2. Boarding/daycare checkout produces an exit-bath or grooming cross-sell opportunity.
3. Staff edits an existing grooming request after new breed, size, coat, matting, product, or history details arrive.
4. A calendar optimizer ranks available groomer windows and needs a duration plus buffer.
5. A groomer reviews service history and proposes a manual duration or rebooking cadence update.

### Actors

- Customer/member: supplies request intent, pet identity, optional preferences, and available windows. They do not approve final internal duration policy.
- Front desk / customer-care staff: captures service intent, confirms missing pet profile facts, and routes review-required estimates.
- Groomer: validates coat condition, matting, style complexity, product needs, handling risk, and manual overrides.
- Manager: approves calendar/buffer overrides, unusual duration compression/extension, repeated no-show impacts, or anything with price/deposit/customer-facing implications.
- AI/estimation agent: drafts predicted duration, confidence, rationale, comparable history evidence, and review flags.
- Deterministic `operations::grooming` policies: own estimate classification, required review level, calendar-safety output, and auditability.
- Neighboring domains: `pet`/`care` provide typed profile facts; `reservation` provides optional add-on context; `customer` provides contact/consent; `money`/`payment` handles quotes/deposits; `workflow`/`tools` executes only approved side effects.

### Inputs

Required typed inputs:

- `entities::LocationId`
- `entities::CustomerId`
- `entities::PetId`
- `operations::grooming::ServiceOffering` or transitional `operations::GroomingService`
- `operations::grooming::Contract` loaded for the location
- A request source such as `CustomerRequest`, `StaffIntake`, `ReservationAddOn`, `CrossSellOpportunity`, or `GroomerReview`

Optional but estimate-shaping inputs:

- `pet::Species`, breed/breed-group, size/weight band, age class, and any provider breed label promoted at the boundary.
- `operations::grooming::CoatCondition` including matting, undercoat, sensitive skin, unknown, or review-required conditions.
- Requested products/options such as deshedding, sensitive-skin shampoo, flea/tick product, medicated product, nail Dremel, ear cleaning, or style complexity.
- `operations::grooming::ServiceHistoryEntry` values with prior duration, groomer, outcome, style notes, product use, and next-cadence recommendation.
- Optional `entities::ReservationId` when the request is linked to boarding/daycare checkout or an add-on.
- Calendar context: preferred/required groomer, requested window, existing calendar blocks, local buffers, and location hours.

### Decisions

The estimate flow must decide:

1. Whether the input is sufficient to produce a policy estimate, an AI-suggested estimate pending review, or a review-only placeholder.
2. Which estimate basis is authoritative: configured breed/coat table, groomer history, location/provider default, manual staff override, or AI suggestion pending review.
3. Whether the estimate requires no review, staff review, groomer review, manager review, or care/medical review.
4. Whether the estimate can be used for calendar search only, draft hold creation, or approved booking execution.
5. Whether extra buffer is needed for matting, undercoat, senior pet handling, sensitive skin/product use, first-time grooming, or unknown data.
6. Whether generated customer text may mention only operational prep details, or must be held for staff/manager approval because it implies suitability, price, deposit, or medical/product advice.

### Outputs

Primary output:

- `operations::grooming::DurationEstimate`: positive `AppointmentMinutes`, optional `BufferMinutes`, `EstimateBasis`, `EstimateConfidence`, `ReviewRequirement`, rationale/evidence, and audit metadata.

Derived outputs:

- `operations::grooming::EstimateDecision`: `UsableForCalendarSearch`, `DraftHoldAllowed`, `ReviewRequired`, or `Rejected`.
- `operations::grooming::ScheduleCandidate` values that include duration, buffer, conflicts, groomer fit, and approval boundary.
- `operations::StaffTask` for groomer/staff/manager review when required.
- `workflow::OperationsAction::SuggestScheduleReview` or `CreateInternalTask` for non-executing workflows.
- Agent result packets that include evidence and review flags, never direct provider mutations.
- Audit events recording who/what produced the estimate and why it was accepted, reviewed, overridden, or rejected.

### Success state

A successful prediction produces a positive, explainable, auditable `DurationEstimate` that can be safely consumed by scheduling policy. If the estimate is high-confidence and policy-allowed, calendar search/ranking can proceed. If review is required, the system still produces a useful draft plan and staff task without booking, charging, messaging, or mutating a reservation.

### Failure and exception states

- `MissingBreedOrCoat`: breed/coat facts are absent or ambiguous. Result: conservative default plus `GroomerReview` or `StaffReview` before booking.
- `UnsupportedSpeciesOrBreed`: the service cannot be estimated for the species/breed/offering. Result: `Rejected` or `ManagerReview` depending on location policy.
- `MattedOrSensitiveCoat`: estimate may be possible but requires groomer/staff review; no auto-booking.
- `MedicalProductOrCareConcern`: grooming references `care` for review; it does not declare medical suitability.
- `NoMatchingPolicyEstimate`: no table/history/default applies. Result: provider/location default with low confidence and review requirement.
- `ConflictingHistory`: previous durations/outcomes disagree materially. Result: explain conflict and require groomer review.
- `CalendarConflict`: estimate is valid, but no candidate window can fit duration plus buffers. Result: schedule review, waitlist, or alternate window draft.
- `ManualOverrideOutOfRange`: staff-entered minutes are zero, implausibly short/long, or conflict with policy. Result: reject or manager review.
- `AiSuggestionUntrusted`: LLM/statistical output lacks evidence, low confidence, or unsafe rationale. Result: draft only and review required.
- `BoundaryConversionFailed`: provider strings/rows cannot be promoted into semantic values. Result: storage/integration error; domain does not receive raw fallback strings.

## 2. Domain types to add or refine

Use semantic paths. Keep the canonical public surface under `operations::grooming`; re-export only when call sites remain truthful. Existing top-level `operations::GroomingService` and `operations::GroomingCadence` can bridge storage compatibility, but grooming-specific behavior should move toward grooming-owned types.

### Core estimate types

- `operations::grooming::DurationEstimate`
  - Fields: `minutes: AppointmentMinutes`, `buffer: Option<BufferMinutes>`, `basis: EstimateBasis`, `confidence: EstimateConfidence`, `review: ReviewRequirement`, `rationale: EstimateRationale`, `evidence: EstimateEvidence`.
  - Invariants: minutes must be positive; buffer must be non-negative through an explicit zero-allowed or positive type; low confidence or missing critical facts cannot have `ReviewRequirement::None`; AI-suggested estimates cannot be `ApprovedForBooking` without an approval token.

- `operations::grooming::AppointmentMinutes`
  - Existing positive scalar; keep rejecting zero.
  - Later policy can bound implausible values through `EstimationPolicy`, not necessarily the scalar itself.

- `operations::grooming::BufferMinutes`
  - Explicit calendar buffer value.
  - Invariant: zero buffer is legal only if represented intentionally (`BufferMinutes::zero_allowed_by(policy)` or `BufferPolicy::NoExtraBuffer`); avoid naked `u16`.

- `operations::grooming::EstimateBasis`
  - Variants: `BreedCoatPolicy`, `GroomerHistory`, `LocationDefault`, `ProviderDefault`, `ManualStaffOverride`, `AiSuggestedPendingReview`.
  - Invariant: `ManualStaffOverride` carries `staff::Id`/`entities::StaffId`, reason, and timestamp; `AiSuggestedPendingReview` carries an agent run/tool trace reference and cannot authorize side effects.

- `operations::grooming::EstimateConfidence`
  - Prefer semantic enum first: `High`, `Medium`, `Low`, `UnknownRequiresReview`.
  - If numerical confidence is needed, add `ConfidenceBasisPoints` bounded 0..=10_000 and wrap it in an enum or struct with an interpretation policy.

- `operations::grooming::ReviewRequirement`
  - Variants: `None`, `StaffReview`, `GroomerReview`, `ManagerReview`, `CareOrMedicalReview`.
  - Invariant: matting, sensitive/medical product needs, unknown breed/coat, first-time complex services, and conflicting history cannot silently become `None`.

- `operations::grooming::EstimateRationale`
  - Trimmed/bounded text or structured enum/value object. Should be staff-facing by default, not member-facing copy.
  - Invariant: generated rationale must cite basis/evidence and must not make medical suitability claims.

- `operations::grooming::EstimateEvidence`
  - Structured evidence: matching policy row, prior history IDs, groomer note refs, provider default version, agent output reference.
  - Invariant: evidence references are typed; do not copy raw provider JSON or sensitive care notes into the estimate.

### Breed, coat, pet-fact types

- `operations::grooming::BreedCategory`
  - Refine existing `ShortCoat`, `DoubleCoat`, `Doodle`, `Cat` into an extensible, policy-driven grouping when implementation needs it: `SmallShortCoat`, `LargeShortCoat`, `DoubleCoat`, `DoodleOrPoodleMix`, `LongCoat`, `Cat`, `UnknownRequiresReview`, `OtherMapped(BreedGroupCode)`.
  - Invariant: unknown or unmapped breed categories carry review requirement; do not fold unknown into a normal category.

- `operations::grooming::CoatCondition`
  - Extend existing `Maintained`, `ThickUndercoat`, `Matted` with `SensitiveSkin`, `FleaTickConcern`, `PostBoardingOdor`, `UnknownRequiresReview`, or model these as separate `CoatAssessment` fields if they can co-exist.
  - Invariant: `Matted`, sensitive-skin/product, and unknown conditions require review before member-facing confirmation.

- `operations::grooming::CoatAssessment`
  - Optional aggregate value containing `condition`, `matting: MattingSeverity`, `undercoat: UndercoatLoad`, `product_needs`, and `assessed_by`.
  - Invariant: if present as groomer-observed, it carries actor/time/source; if customer-reported, it is lower trust and may require validation.

- `operations::grooming::PetEstimateProfile`
  - Grooming-facing projection of `pet`/`care` facts: pet ID, species, breed category, optional size band, coat assessment, relevant care review refs, and history summary.
  - Invariant: owns no customer identity, medical diagnosis, or free-form duplicated care facts; it references care/medical concerns through typed refs.

- `operations::grooming::ProductNeed`
  - Variants: `Deshedding`, `SensitiveSkinShampoo`, `MedicatedProductRequiresStaffReview`, `FleaTickProductRequiresPolicyReview`, `Other(ProductInstruction)`.
  - Invariant: medical or pesticide-sensitive products route to review; product need can extend time but cannot claim care suitability.

### Request, decision, and calendar bridge types

- `operations::grooming::AppointmentRequest`
  - Required fields: location, customer, pet, requested offering, source.
  - Optional fields: reservation link, requested window, preferred/required groomer, style/product notes, first-time flag.
  - Invariant: construction through builder requires the semantic IDs and exact offering; provider strings are converted before construction.

- `operations::grooming::EstimateRequest`
  - A narrower request for estimation: `AppointmentRequest`, `PetEstimateProfile`, optional `ServiceHistorySummary`, `Contract`, and caller intent (`CalendarSearch`, `DraftHold`, `StaffReview`).
  - Invariant: has enough context to explain why an estimate can/cannot be trusted.

- `operations::grooming::EstimateDecision`
  - Variants: `UsableForCalendarSearch(DurationEstimate)`, `DraftHoldAllowed(DurationEstimate)`, `ReviewRequired { estimate, task }`, `Rejected { reason }`.
  - Invariant: review-required decisions carry a specific review owner/reason; rejection carries semantic denial reason, not a string.

- `operations::grooming::ScheduleCandidate`
  - Candidate groomer/window/duration/buffer/conflict explanation.
  - Invariant: candidate never overlaps existing blocks unless represented as `ManagerOverrideRequired`.

- `operations::grooming::ManualEstimateOverride`
  - Staff/groomer/manager override with actor, minutes, reason, source estimate, timestamp, and approval level.
  - Invariant: cannot erase original estimate/audit trail; cannot reduce review level below policy without manager approval.

### Audit and error types

- `operations::grooming::EstimationAuditEvent`
  - Events: `Estimated`, `ReviewRequired`, `StaffAccepted`, `GroomerAdjusted`, `ManagerApprovedOverride`, `Rejected`, `BoundaryConversionFailed`.
  - Invariant: records actor/source and typed reason without storing secrets or raw PII-heavy payloads.

- `operations::grooming::EstimationError`
  - Module-local error enum with `Result<T>` alias.
  - Variants should name semantic failures: `MissingRequiredPetProfile`, `InvalidManualOverride`, `NoPolicyEstimate`, `UnsupportedOffering`, `BoundaryPromotionFailed`, `UnsafeForAutomation`.

## 3. Relationship map between types

### Entities and value objects

- `entities::LocationId`, `entities::CustomerId`, `entities::PetId`, `entities::ReservationId`, and `entities::StaffId` identify the operational context but remain owned by `entities`.
- `operations::grooming::AppointmentRequest` binds location/customer/pet/offering/source into a grooming request.
- `operations::grooming::PetEstimateProfile` is a grooming projection of pet facts. It references `pet`/`care` facts instead of owning them.
- `operations::grooming::BreedCategory`, `CoatCondition`, `CoatAssessment`, `ProductNeed`, `AppointmentMinutes`, `BufferMinutes`, `EstimateConfidence`, and `EstimateRationale` are value objects/semantic enums.
- `operations::grooming::DurationEstimate` is the central value produced by policies and consumed by scheduling.

### Policies and domain services

- `operations::grooming::EstimationPolicy` owns estimate calculation, review classification, basis selection, and rationale composition.
- `operations::grooming::CalendarPolicy` remains the location-level calendar rule. `SchedulingPolicy` consumes `DurationEstimate`; it should not recalculate breed/coat duration from raw facts.
- `operations::grooming::AutomationPolicy` maps estimate decisions and actions to draft/staff/manager/disallowed boundaries.
- `operations::grooming::HistoryPolicy` decides how completed appointments become evidence for future estimates.
- `operations::grooming::NoShowPolicyEngine` may add deposit/manager review constraints, but it should not own duration math.

### Repositories and stores

- `operations::grooming::ContractRepository` loads location-specific policy tables and defaults.
- `operations::grooming::EstimatePolicyRepository` loads breed/coat/service estimate rules as semantic rows, not provider strings.
- `operations::grooming::HistoryRepository` reads prior grooming history and appends staff-reviewed history entries.
- `operations::grooming::CalendarRepository` reads blocks and creates draft holds only when approval policy permits; final booking belongs behind an approved tool port.
- `storage::operations::*Record` stores provider/storage-compatible shapes and converts them into domain types at boundaries.

### Workflow events, staff tasks, and agent outputs

- `workflow::WorkflowEventId` can source staff tasks and audit links.
- Existing `operations::StaffTaskKind::{DocumentReview, CustomerFollowUp}` can be used only if truthful. Add grooming-specific variants once needed: `GroomingEstimateReview`, `GroomingHistoryReview`, `GroomingCalendarOverrideReview`.
- Staff task assignments should use `operations::StaffTaskAssignment::Role(StaffRole::Groomer)` or `Role(StaffRole::Manager)` when no exact staff ID is selected.
- Agent specs/tools should expose a `grooming_time_estimator` or `GroomingDurationEstimateAgent` returning typed `DurationEstimateSuggestion` with evidence, risk flags, confidence, and review requirement.
- Tools such as Gingr/calendar adapters execute provider actions only from approved command values, never directly from an agent suggestion.

### Truthful ownership rule

- Breed/coat duration prediction belongs to `operations::grooming::EstimationPolicy`, not to generic helpers, calendar repositories, AI prompt code, or storage codecs.
- Calendar fit belongs to `operations::grooming::SchedulingPolicy`/calendar repository, not to the estimate object.
- Care/medical suitability belongs to `care`/staff review; grooming may require review based on a reference, but it does not make medical claims.
- Customer communication consent belongs to `customer`/messaging policy; grooming may draft reasons and prep notes, not send.

## 4. Interaction contract

Rust-like pseudo-signatures below describe domain contracts, not exact implementation requirements.

```rust
pub mod operations::grooming {
    pub struct EstimateRequest {
        pub appointment: AppointmentRequest,
        pub pet_profile: PetEstimateProfile,
        pub history: ServiceHistorySummary,
        pub contract: Contract,
        pub intent: EstimateIntent,
    }

    pub enum EstimateIntent {
        CalendarSearch,
        DraftHold,
        StaffReview,
        ReestimateExistingAppointment { appointment_id: AppointmentId },
    }

    pub struct DurationEstimate {
        pub minutes: AppointmentMinutes,
        pub buffer: Option<BufferMinutes>,
        pub basis: EstimateBasis,
        pub confidence: EstimateConfidence,
        pub review: ReviewRequirement,
        pub rationale: EstimateRationale,
        pub evidence: EstimateEvidence,
    }

    pub enum EstimateDecision {
        UsableForCalendarSearch(DurationEstimate),
        DraftHoldAllowed(DurationEstimate),
        ReviewRequired {
            estimate: DurationEstimate,
            task: StaffTaskDraft,
            reason: ReviewReason,
        },
        Rejected { reason: EstimationDenialReason },
    }
}
```

### `EstimationPolicy`

Owner: `operations::grooming::EstimationPolicy`.

```rust
impl EstimationPolicy {
    pub fn estimate(&self, request: EstimateRequest) -> Result<EstimateDecision>;

    pub fn classify_review(
        &self,
        request: &EstimateRequest,
        estimate: &DurationEstimate,
    ) -> ReviewRequirement;

    pub fn apply_manual_override(
        &self,
        current: DurationEstimate,
        override_: ManualEstimateOverride,
    ) -> Result<DurationEstimate>;
}
```

Behavior contract:

- Selects the strongest available basis in this order unless location policy says otherwise: valid groomer history, exact breed/coat/service policy row, location default, provider default, AI suggestion pending review.
- Produces positive minutes and explicit confidence/review.
- Routes `UnknownRequiresReview`, matting, sensitive/medical product needs, conflicting history, first-time complex services, and unsupported species to review/rejection.
- Does not query storage or execute tools directly; callers provide repositories or snapshots.
- Does not silently normalize a low-confidence AI estimate into a policy estimate.

### `EstimatePolicyRepository`

Owner: repository/store port under `operations::grooming` with storage adapter implementation elsewhere.

```rust
#[async_trait]
pub trait EstimatePolicyRepository {
    async fn load_for_location(
        &self,
        location_id: entities::LocationId,
    ) -> Result<EstimatePolicyTable>;
}

pub struct EstimatePolicyRow {
    pub offering: ServiceOffering,
    pub breed: BreedCategory,
    pub coat: CoatCondition,
    pub base_minutes: AppointmentMinutes,
    pub buffer: Option<BufferMinutes>,
    pub review_rule: ReviewRule,
}
```

Behavior contract:

- Promotes provider/location codes into semantic rows before returning them.
- Rejects zero-minute rows and unknown raw service names at the boundary.
- Supports extension rows without turning the domain into `String`ly typed maps.

### `HistoryRepository` and `HistoryPolicy`

```rust
#[async_trait]
pub trait HistoryRepository {
    async fn load_summary_for_pet(
        &self,
        pet_id: entities::PetId,
        location_id: entities::LocationId,
    ) -> Result<ServiceHistorySummary>;
}

impl HistoryPolicy {
    pub fn estimate_evidence_from_history(
        &self,
        history: &ServiceHistorySummary,
        requested: ServiceOffering,
    ) -> Option<EstimateEvidence>;
}
```

Behavior contract:

- History evidence uses completed, staff-reviewed grooming history.
- Style notes and care/medical refs stay separated.
- Prior duration can influence a new estimate, but conflicting outcomes require groomer review instead of averaging away the conflict.

### `SchedulingPolicy` and `CalendarRepository`

```rust
impl SchedulingPolicy {
    pub fn rank_windows(
        &self,
        request: &AppointmentRequest,
        estimate: &DurationEstimate,
        calendar: GroomerCalendarSnapshot,
        contract: &Contract,
    ) -> Vec<ScheduleCandidate>;
}

#[async_trait]
pub trait CalendarRepository {
    async fn load_snapshot(
        &self,
        location_id: entities::LocationId,
        window: ScheduleWindow,
    ) -> Result<GroomerCalendarSnapshot>;

    async fn create_draft_hold(
        &self,
        candidate: ScheduleCandidate,
        approval: DraftHoldApproval,
    ) -> Result<DraftHoldRef>;
}
```

Behavior contract:

- Scheduling consumes `DurationEstimate`; it does not inspect raw breed/coat values.
- Candidate windows include buffers and conflict explanations.
- Confirmed booking is not a repository method here; it belongs behind an approved tool command after review/approval.

### `AutomationPolicy`

```rust
impl AutomationPolicy {
    pub fn decide_estimate_use(
        &self,
        decision: &EstimateDecision,
        action: GroomingAction,
        actor: ActorContext,
    ) -> AutomationDecision;
}
```

Behavior contract:

- Calendar search and ranking may be automated from high-confidence policy estimates.
- Draft holds may be allowed only when policy and location rules permit.
- Booking, rescheduling, customer messaging, deposits, discounts, and reservation add-ons require staff/manager approval or are disallowed.
- AI output is never an approval token.

### Agent/tool contract

```rust
pub trait GroomingDurationEstimateAgent {
    fn suggest_duration(
        &self,
        packet: GroomingEstimatePromptPacket,
    ) -> AgentResult<DurationEstimateSuggestion>;
}

pub struct DurationEstimateSuggestion {
    pub suggested_minutes: AppointmentMinutes,
    pub confidence: EstimateConfidence,
    pub rationale: EstimateRationale,
    pub evidence: EstimateEvidence,
    pub risk_flags: Vec<EstimateRiskFlag>,
}
```

Behavior contract:

- Agent packets use typed fields and redacted/referenced sensitive facts.
- Agent output is converted through `EstimationPolicy` before use.
- Tools receive only approved commands such as `CreateDraftHold`, `CreateStaffTask`, or `PrepareCustomerMessageDraft`; no free-form tool command may book or message.

## 5. Review and approval contract

### Automation level

Allowed without human review:

- Load estimate rules/history/calendar snapshots.
- Produce a draft `DurationEstimate` for staff/internal use.
- Rank calendar windows for internal review when the estimate has policy/high confidence and no safety flags.
- Create internal draft artifacts or non-member-facing summaries.

Staff or groomer review required:

- Missing/unknown breed, coat, size, or first-time service data.
- Matted coat, thick undercoat with uncertain time, sensitive skin, flea/tick concern, medical-product/product suitability concerns, or handling/care refs.
- Style complexity or groomer-history conflict that could materially change appointment time.
- AI/statistical estimate with low/medium confidence, missing evidence, or risk flags.
- Any generated service-history interpretation before it becomes durable evidence.

Manager review required:

- Calendar double-booking, buffer compression, groomer-specific override, or capacity exception.
- Manual override outside normal policy bounds or override that reduces required review level.
- Rebooking restrictions/deposit requirements tied to repeated no-shows or late cancellations.
- Price, deposit, refund, discount, complaint, injury/safety, or legal/reputation implications.

Disallowed without explicit approval token:

- Confirming/rescheduling/cancelling a grooming appointment in a provider system.
- Adding grooming services/products to a reservation/invoice.
- Charging, refunding, waiving, or demanding deposits/discounts.
- Sending customer/member-facing confirmations, prep instructions, estimates, or medical/product suitability claims.
- Rewriting or hiding concerning groomer notes, incident facts, or care-sensitive history.

### Audit trail

Every accepted/reviewed/overridden estimate should record:

- Estimate ID or appointment/request ID.
- Location, pet, customer, optional reservation, and requested offering IDs.
- Source actor: customer intake, staff, groomer, manager, provider default, policy table, or agent run.
- Input snapshot references: policy version, history entry IDs, care refs, calendar snapshot ID.
- Output: minutes, buffer, confidence, review requirement, rationale, basis.
- Decision: draft, accepted for calendar search, draft hold, review required, rejected, manager approved.
- Override metadata when present: actor, previous estimate, new estimate, reason, timestamp.

### Customer/member-facing boundaries

A duration estimate can inform internal scheduling and staff preparation. It should not be presented to a customer as a guaranteed completion time, price, product suitability, or medical/care statement unless an explicit member-facing message policy approves the wording. Customer-facing text should say the appointment plan is pending groomer/location confirmation when review is required.

## 6. Test contracts

Semantic domain tests:

1. `grooming_estimation_policy_returns_policy_duration_for_known_breed_coat_service`
   - Known offering + breed category + coat condition returns positive minutes, `EstimateBasis::BreedCoatPolicy`, and no review when policy permits.

2. `grooming_duration_estimate_rejects_zero_minutes_and_requires_basis_confidence_and_review`
   - `DurationEstimate` cannot exist with zero minutes or missing basis/confidence/review semantics.

3. `grooming_unknown_breed_or_coat_produces_review_required_not_silent_default`
   - Unknown breed/coat can use a conservative default only with `StaffReview`/`GroomerReview` and rationale.

4. `grooming_matted_or_sensitive_coat_requires_groomer_review_before_booking`
   - Matted/sensitive/product-sensitive cases return review-required and cannot authorize booking.

5. `grooming_history_based_estimate_carries_prior_service_evidence`
   - Prior completed service can become `EstimateBasis::GroomerHistory` only when evidence points to typed history entries.

6. `grooming_conflicting_history_requires_review_instead_of_averaging_duration`
   - Materially conflicting prior durations/outcomes produce groomer review and conflict rationale.

7. `grooming_manual_override_preserves_original_estimate_actor_reason_and_audit_event`
   - Staff/groomer override cannot erase source estimate and must carry actor/reason/time.

8. `grooming_manager_review_required_for_calendar_buffer_or_double_booking_override`
   - Scheduling cannot compress buffers or overlap blocks without manager-review decision.

9. `grooming_ai_duration_suggestion_maps_to_pending_review_without_approval_token`
   - AI output can draft a suggestion but cannot become approved tool execution by itself.

10. `grooming_schedule_candidate_uses_duration_estimate_without_recalculating_breed_coat_logic`
    - Scheduling policy consumes estimate and calendar data; breed/coat math stays in estimation policy.

11. `grooming_estimate_request_promotes_provider_breed_and_service_strings_at_boundary`
    - Storage/integration raw values either convert into semantic enums/newtypes or produce boundary promotion errors.

12. `grooming_member_facing_message_for_review_required_estimate_is_draft_only`
    - Customer-facing wording is a draft requiring approval when estimate review is required.

Storage/boundary tests:

13. `grooming_estimate_policy_records_reject_zero_minutes_and_unknown_raw_service_code`
14. `grooming_estimate_policy_records_roundtrip_known_breed_coat_service_rows`
15. `grooming_history_records_keep_style_notes_separate_from_care_refs_for_estimate_evidence`
16. `grooming_calendar_adapter_creates_draft_hold_not_confirmed_booking_from_estimate_decision`

Workflow/agent tests:

17. `grooming_time_estimator_agent_returns_evidence_confidence_and_risk_flags`
18. `grooming_estimation_workflow_creates_groomer_review_task_for_unknown_or_matted_coat`
19. `grooming_estimation_workflow_never_sends_customer_message_or_books_provider_slot_from_llm_suggestion_alone`
20. `grooming_estimate_audit_event_records_override_without_raw_sensitive_payloads`

## 7. Integration notes for later serialized Rust code card

Likely files touched:

- `domain/src/operations.rs`
  - Add/refine `operations::grooming` estimate types, policy structs, decision enums, and module-local errors.
  - Consider splitting the large file into `domain/src/operations/grooming.rs` only if the card includes module restructuring; otherwise keep a narrow addition to avoid scope creep.

- `domain/tests/petsuites_core_service_contracts.rs`
  - Extend existing grooming contract tests with duration estimate semantics.

- `domain/tests/domain_quality_patterns.rs`
  - Add semantic-pattern tests if they already enforce no raw helper/primitive patterns.

- Storage crate/tests, if present in the later card:
  - Expand `storage/tests/core_service_contract_storage.rs` and `storage/tests/operations_storage_contracts.rs` for estimate policy rows and cross-variant rejection.

- Workflow/agent/tool modules, only after domain types exist:
  - `domain/src/agents.rs`, `domain/src/workflow.rs`, `domain/src/tools.rs`, or dedicated adapter crates for typed agent result/tool command surfaces.

Migration/refactor risks:

- There are currently two cadence concepts: top-level `operations::CadenceWeeks`/`GroomingCadence` and nested `operations::grooming::CadenceWeeks`/`RebookingCadence`. Do not add a third cadence shape for estimates; choose one owner when touching cadence behavior.
- Current `BreedCoatTimeEstimate` only stores `BreedCategory`, `CoatCondition`, and minutes. Extending it in place may break serialization. Prefer adding `DurationEstimate` and preserving `BreedCoatTimeEstimate` as a simple policy-row input until storage migration is planned.
- Expanding `BreedCategory`/`CoatCondition` enum variants can affect serde round-trips. Add storage compatibility tests before changing serialized variants.
- `operations::StaffTaskKind` lacks grooming-specific review variants. Use existing `DocumentReview`/`CustomerFollowUp` only when truthful; otherwise add explicit grooming task variants in the same code card as tests.
- Avoid letting AI-agent packet schemas become the canonical domain model. Agent outputs should convert into `DurationEstimateSuggestion` and then pass through deterministic `EstimationPolicy`.
- Keep care/medical details as refs/review flags. Do not copy allergies, medical conditions, or sensitive handling notes into generic grooming estimate rationale.
- Do not couple estimate approval to member-facing messaging. Approval for calendar search is weaker than approval to book, charge, or message.

Dependencies on other implications/cards:

- Grooming service menu/offering modeling should settle whether `operations::grooming::ServiceOffering` replaces or wraps top-level `operations::GroomingService`.
- Groomer calendar/scheduling implication should consume `DurationEstimate` and own conflict/buffer/window ranking.
- Grooming service history implication should define `ServiceHistoryEntry`, `StyleNote`, `HandlingNoteRef`, and history evidence IDs.
- No-show/cancellation/deposit implication should own deposit/manager-review constraints after repeated no-shows; estimation only reports scheduling duration and review requirements.
- Reminder/rebooking/customer communication implications should own member-facing message approvals and consent checks.

Implementation entry rule:

Write the failing semantic API tests first. Start with `grooming_duration_estimate_rejects_zero_minutes_and_requires_basis_confidence_and_review` and `grooming_unknown_breed_or_coat_produces_review_required_not_silent_default`, then implement the smallest `operations::grooming` types and `EstimationPolicy` necessary to make those truths compile and pass. Do not add provider booking, customer messaging, or AI tool execution in the first estimate-domain slice.
