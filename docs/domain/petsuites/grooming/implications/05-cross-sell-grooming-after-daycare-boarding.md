# Grooming implication 05: Cross-sell grooming after daycare/boarding

Purpose: model the operational implication that grooming should be offered after daycare or boarding when it is safe, relevant, available, and reviewable. This is a domain contract for later Rust implementation, not an instruction to auto-sell, auto-book, charge, or message members.

Assumptions:

- PetSuites commonly exposes grooming as both a standalone service and an add-on/cross-sell to boarding/daycare reservations.
- The safest v1 model is recommendation-first: agents and policies may identify an opportunity, draft a message, or create a staff task, but cannot mutate a reservation, book a groomer calendar, apply a charge, or send a customer message without the appropriate approval boundary.
- Exit bath after boarding is the canonical low-complexity offer. Full groom, premium bath, nail/ear services, first-time offers, or product-specific recommendations require stronger menu, eligibility, duration, calendar, and review checks.
- If care/medical/handling facts are relevant, grooming references the owning care/pet facts and asks for staff review; it does not duplicate sensitive medical conclusions in generic sales notes.

## 1. Operational story

### Trigger

A `reservation` or daily operations workflow emits one of these typed trigger contexts:

- `operations::grooming::cross_sell::Trigger::BoardingCheckoutApproaching` when a boarding stay is nearing checkout and an exit bath or grooming slot might improve the checkout experience.
- `operations::grooming::cross_sell::Trigger::DaycareVisitCompleted` when a daycare visit creates a natural follow-up opportunity for bath, nail, ear, or first-time grooming.
- `operations::grooming::cross_sell::Trigger::PostStayFollowUpDue` when the customer should receive a post-stay check-in and grooming can be offered as a reviewed draft.
- `operations::grooming::cross_sell::Trigger::LapsedGroomingCadenceObserved` when grooming history shows the pet is due/overdue and the current boarding/daycare relationship gives staff a timely reason to follow up.

The trigger must carry typed source identity and timing, not raw provider strings:

```rust
enum Trigger {
    BoardingCheckoutApproaching {
        reservation_id: reservation::Id,
        checkout_window: reservation::CheckoutWindow,
    },
    DaycareVisitCompleted {
        reservation_id: reservation::Id,
        visit_date: calendar::ServiceDate,
    },
    PostStayFollowUpDue {
        reservation_id: reservation::Id,
        due_at: workflow::DueAt,
    },
    LapsedGroomingCadenceObserved {
        reservation_id: Option<reservation::Id>,
        recommendation: grooming::RebookingRecommendation,
    },
}
```

Use current `entities::ReservationId` until `reservation::Id` is canonical.

### Actors

- Front desk staff: reviews and sends customer-facing grooming offers; may add approved add-ons to checkout/reservation workflows.
- Groomer: reviews service suitability, duration, style/history ambiguity, and calendar fit.
- Manager: approves exceptions, discounts/first-time offer ambiguity, repeated no-show/deposit handling, calendar overrides, or high-risk care/complaint scenarios.
- Customer/member: receives only approved messages/offers and explicitly accepts or declines.
- Pet: represented through typed pet identity and care/profile facts; the pet cannot be treated as a generic reservation line item.
- AI/grooming agent: may rank opportunities, draft rationale/messages, summarize evidence, and propose staff tasks. It never grants execution permission.

### Inputs

Required inputs:

- Source reservation identity: `reservation::Id` / current `entities::ReservationId`.
- `entities::LocationId`, `entities::CustomerId`, and `entities::PetId` promoted from reservation/customer/pet repositories.
- Source service context: `operations::grooming::cross_sell::SourceServiceContext` distinguishing boarding checkout, daycare visit, post-stay follow-up, and lapsed cadence.
- Location grooming contract: `operations::grooming::Contract` loaded per location.
- Location service menu: `operations::grooming::ServiceMenu` or transitional mapping from top-level `operations::GroomingService` variants.
- Pet profile/care facts needed for eligibility, breed/coat estimation, and sensitive product review.
- Grooming service history: prior offerings, style notes, cadence, no-show/cancellation history, product-use outcomes, and next due window.
- Groomer calendar snapshot if the offer implies same-day checkout scheduling or an appointment proposal.
- Customer communication consent/preferences and channel policy.
- Current reservation/add-on/payment state so the opportunity does not duplicate an already-selected service or create an unapproved charge.

Optional inputs:

- Revenue opportunity context from `operations::RevenueOpportunityKind::ExitBathAfterBoarding` or a future `GroomingAfterDaycare` / `GroomingCrossSell` variant.
- AI evidence packet: opportunity score, natural-language rationale, matching prior history, capacity/calendar explanation, and risk flags.
- Promotional/first-time offer rules from an offer repository.

### Decisions

The truthful owner for cross-sell decisions is `operations::grooming::CrossSellPolicy`, supported by menu, history, estimation, calendar, no-show, and automation policies.

Policy decisions:

1. Is grooming eligible to be suggested from this source context?
   - Boarding checkout may suggest `ExitBath` by default when menu-enabled and not already selected.
   - Daycare may suggest bath/nail/ear/first-time offer only when menu-enabled and the recommendation has a staff-reviewable reason.
   - Lapsed cadence may suggest rebooking if history indicates due/overdue.
2. Is the pet/customer/reservation context safe for a grooming offer?
   - Missing pet profile, matted/sensitive/medical-product cases, handling concerns, or care ambiguity force staff/groomer review.
   - Repeated no-show/late cancellation or deposit restrictions force manager review or a deposit-required draft.
3. Is there a viable operational path?
   - Same-day boarding exit bath needs a compatible checkout window and groomer/capacity availability or an explicit staff task to check availability.
   - Full groom and groomer-specific services require duration estimate and calendar candidate review.
4. What is the correct output boundary?
   - Internal recommendation only.
   - Staff task for front desk/groomer review.
   - Customer message draft requiring approval.
   - Appointment/add-on draft requiring customer acceptance and tool approval.
   - Disallowed/withheld with a typed reason.

### Outputs

Primary output:

```rust
struct CrossSellOpportunity {
    id: cross_sell::OpportunityId,
    source: cross_sell::SourceServiceContext,
    customer_id: entities::CustomerId,
    pet_id: entities::PetId,
    location_id: entities::LocationId,
    recommended_offering: grooming::ServiceOffering,
    recommendation: cross_sell::RecommendationKind,
    rationale: cross_sell::Rationale,
    evidence: cross_sell::Evidence,
    review: grooming::ReviewRequirement,
    automation: grooming::AutomationBoundary,
    next_action: cross_sell::NextAction,
}
```

Secondary outputs:

- `operations::RevenueOpportunity` for manager/daily brief surfaces, preserving the current `RevenueOpportunityKind::ExitBathAfterBoarding` integration point.
- `operations::StaffTask` with `StaffTaskKind::CustomerFollowUp`, `DocumentReview`, or future grooming-specific variants such as `GroomingCrossSellReview` and `GroomerCalendarReview`.
- `workflow::RecommendedAction` / current `operations::OperationsAction::SuggestRevenueFollowUp` or `DraftCustomerMessage`.
- `grooming::AppointmentRequest` or `reservation::AddOnDraft`, but never a booked appointment or committed add-on without approval and acceptance.
- `grooming::cross_sell::DeclinedOpportunity` / `SuppressedOpportunity` for audit and future data-quality review when no offer should be made.

### Success state

A successful cross-sell evaluation does not mean revenue was captured. It means the system produced the safest truthful next step:

- an eligible, non-duplicative opportunity linked to the source reservation and pet/customer/location;
- an explicit recommended grooming offering and rationale;
- any care/history/calendar/no-show risks encoded as review requirements;
- no reservation mutation, payment mutation, appointment booking, or member-facing send unless a typed approval/acceptance boundary is present;
- an auditable record of what was recommended, withheld, drafted, approved, declined, or executed.

### Failure and exception states

Represent exceptions as semantic outcomes, not booleans or logs:

- `SourceReservationNotEligible`: reservation is cancelled, checked out too late, already has equivalent grooming, or source service is unknown.
- `MenuOfferingUnavailable`: location does not enable the recommended grooming service.
- `CustomerConsentMissing`: a customer-facing draft may be prepared internally but cannot be sent.
- `PetProfileIncomplete`: missing species/breed/size/coat facts needed for safe estimate or suitability.
- `CareReviewRequired`: allergies, medical product, sensitive skin, incident, or handling note makes offer unsafe without care/staff review.
- `CalendarUnavailable`: no same-day groomer capacity; produce future appointment follow-up or staff task instead.
- `NoShowOrDepositRestriction`: no-show/late-cancel history requires deposit/manager review before rebooking.
- `OfferAlreadySelected`: avoid duplicate add-ons or confusing duplicate messages.
- `FirstTimeOfferAmbiguous`: staff/manager review required before promotional language.
- `AutomationDisallowed`: attempted booking/payment/message execution without approval token.

## 2. Domain types to add or refine

Prefer a grooming-owned submodule so call sites preserve the implication:

```rust
operations::grooming::cross_sell
```

### Core entities and value objects

- `operations::grooming::cross_sell::OpportunityId`
  - Non-empty provider/domain ID or UUID wrapper. Provider IDs are promoted at storage/tool boundaries.
- `operations::grooming::CrossSellOpportunity`
  - Entity-like recommendation aggregate. It is not a booking and not a reservation add-on.
  - Invariants: one source context, one customer, one pet, one location, one recommended offering, one automation boundary, at least one evidence item.
- `operations::grooming::cross_sell::SourceServiceContext`
  - Enum: `BoardingCheckout { reservation_id, checkout_window }`, `BoardingPostStay { reservation_id }`, `DaycareVisit { reservation_id, visit_date }`, `LapsedCadence { recommendation }`, `StaffPrompted { staff_id }`.
  - Invariant: source reservation IDs must be typed; daycare and boarding are distinct variants even if both are reservations.
- `operations::grooming::cross_sell::RecommendationKind`
  - Enum: `ExitBathBeforeCheckout`, `BathAfterDaycare`, `NailOrEarCareFollowUp`, `FirstTimeGroomingOffer`, `FullGroomConsult`, `RebookingDue`, `StaffSuggestedAlternative`.
  - Invariant: do not use a broad `Grooming` string when exact offer affects review/calendar/payment.
- `operations::grooming::ServiceOffering`
  - Grooming-owned enum from the service map: `MiniGroom`, `FullGroom`, `ExitBath`, `FullBath`, `PremiumBath`, `NailTrim`, `NailDremel`, `EarCleaning`, `CoatSkinSpecificProduct`, `FirstTimeOffer`.
  - Transitional adapter can map from current `operations::GroomingService`.
- `operations::grooming::ServiceMenu`
  - Non-empty location menu; rejects duplicate offerings; carries offer availability/eligibility metadata.
- `operations::grooming::cross_sell::Rationale`
  - Trimmed bounded text, ideally structured by reason codes plus display copy. Do not store raw LLM prose as the only reason.
- `operations::grooming::cross_sell::Evidence`
  - Typed enum/list: `PriorServiceHistory`, `DueForCadence`, `BoardingCheckoutTiming`, `DaycareVisitRecency`, `MenuEnabled`, `CalendarCandidateAvailable`, `CustomerRequestedSimilarService`, `StaffNoteReference`, `AiSuggestedPendingReview`.
  - Invariant: at least one non-AI evidence item before customer-facing drafts are eligible for staff review.
- `operations::grooming::cross_sell::SuppressionReason`
  - Semantic reason for withholding: duplicate, no consent, medical/care review, no capacity, unavailable service, no-show restriction, low confidence.
- `operations::grooming::cross_sell::NextAction`
  - Enum: `NoAction`, `CreateStaffTask`, `DraftCustomerMessage`, `DraftAddOnForCustomerAcceptance`, `DraftAppointmentRequest`, `ManagerReview`, `Suppress`.

### Policy/refinement types

- `operations::grooming::CrossSellPolicy`
  - Owns eligibility and opportunity selection from boarding/daycare context.
- `operations::grooming::CrossSellDecision`
  - Enum: `Opportunity(CrossSellOpportunity)`, `Suppressed(SuppressedOpportunity)`, `NeedsReview(ReviewRequest)`.
- `operations::grooming::ReviewRequirement`
  - Refine/introduce: `None`, `FrontDeskReview`, `GroomerReview`, `ManagerReview`, `CareOrMedicalReview`, `CustomerAcceptanceRequired`.
- `operations::grooming::AutomationBoundary`
  - Refine/introduce: `InternalRecommendationOnly`, `StaffApprovalRequired`, `GroomerApprovalRequired`, `ManagerApprovalRequired`, `MemberFacingSendRequiresApproval`, `ToolExecutionRequiresApproval`, `Disallowed`.
- `operations::grooming::ApprovalToken`
  - Opaque typed proof of human approval for a specific action, actor, time, and opportunity. It should not be forgeable from an LLM response.
- `operations::grooming::DurationEstimate`
  - Required when the cross-sell can become an appointment request rather than just an exit-bath follow-up.
- `operations::grooming::CalendarCandidate`
  - Represents a possible appointment/hold; not a booking.
- `operations::grooming::AppointmentRequest`
  - Draft request created only after policy says the offering can be proposed; customer acceptance and approval still gate scheduling.
- `reservation::AddOnDraft` or `operations::grooming::ReservationAddOnDraft`
  - If reservation domain is not ready, keep a grooming-side draft command that names the intended reservation mutation but cannot commit it.

### Repository/store contracts

- `operations::grooming::Repository`
  - Aggregate read/write for grooming domain forms.
- `operations::grooming::ContractRepository`
  - Loads the grooming contract snapshot for location.
- `operations::grooming::OfferRepository`
  - Loads `ServiceMenu`, first-time offer eligibility, and location-specific availability.
- `operations::grooming::HistoryRepository`
  - Reads prior service history, cadence, no-show/late-cancel history, and prior cross-sell outcomes.
- `operations::grooming::CalendarRepository`
  - Reads availability and may create draft holds only through an approved tool boundary.
- `operations::grooming::CrossSellRepository`
  - Persists opportunities, suppressions, approvals, declines, customer acceptances, and execution outcomes for audit.

### Agent/task/tool contracts

- `operations::grooming::agent::CrossSellSpec`
  - Prompt/input packet schema for AI ranking/drafting. It receives typed evidence and returns typed recommendations with confidence and risk flags.
- `operations::grooming::agent::CrossSellDraft`
  - AI output that cannot be executed directly. It must be promoted through deterministic policy.
- `workflow::task::Kind` / `operations::StaffTaskKind`
  - Add future variants only when current `CustomerFollowUp`/`DocumentReview` are too vague: `GroomingCrossSellReview`, `GroomerCalendarReview`, `GroomingOfferApproval`.
- Tool ports
  - `CreateGroomingCrossSellTask`, `DraftCustomerMessage`, `CreateReservationAddOnDraft`, `CreateAppointmentHold`, `RecordCrossSellOutcome`. All are boundary commands requiring authorization appropriate to their side effects.

## 3. Relationship map between types

### Entities

- `grooming::CrossSellOpportunity` links one `customer::Id`, one `pet::Id`, one `location::Id`, and zero/one `reservation::Id` source.
- `reservation::Reservation` remains the owner of boarding/daycare lifecycle and add-on mutation. Grooming records a proposed add-on/request, not the committed reservation state.
- `grooming::AppointmentRequest` is a draft scheduling entity; `grooming::AppointmentPlan`/`AppointmentId` own appointment-specific state later.
- `grooming::ServiceHistoryEntry` feeds lapsed-cadence and prior-service evidence but remains separate from opportunity identity.

### Value objects

- `grooming::ServiceOffering` identifies the exact service being recommended.
- `cross_sell::SourceServiceContext` preserves why this is a boarding/daycare cross-sell rather than a generic grooming lead.
- `cross_sell::Rationale`, `cross_sell::Evidence`, and `cross_sell::Confidence` explain why the recommendation exists.
- `grooming::ReviewRequirement`, `grooming::AutomationBoundary`, and `grooming::ApprovalToken` encode action legality.
- `grooming::DurationEstimate`, `ScheduleWindow`, and `CalendarCandidate` support appointment-shaped offers.

### Policies

- `CrossSellPolicy` is the primary owner for choosing/suppressing opportunities.
- `EstimationPolicy` owns duration and staff-review implications for service/breed/coat/history.
- `SchedulingPolicy` owns calendar fit and no-overlap/buffer rules.
- `NoShowPolicyEngine` owns deposit/rebooking restrictions.
- `ReminderPolicy` or `CommunicationPolicy` owns customer message timing/consent gates.
- `AutomationPolicy` owns promotion from recommendation/draft to staff approval, manager approval, disallowed, or tool-executable.

### Repositories/stores

- `ContractRepository` and `OfferRepository` load location policy/menu snapshots.
- `HistoryRepository` loads grooming history and prior cross-sell outcomes.
- `CalendarRepository` loads groomer availability and creates only approved draft holds.
- `reservation::Repository` loads source reservation facts and remains owner of committed add-ons.
- `CrossSellRepository` persists opportunity lifecycle and audit trail.
- Storage records remain boundary shapes: raw provider IDs, menu codes, message IDs, and provider reservation payloads must promote into semantic types before policy runs.

### Workflow events

Suggested event vocabulary:

- `grooming::event::CrossSellOpportunityIdentified`
- `grooming::event::CrossSellOpportunitySuppressed`
- `grooming::event::CrossSellReviewRequested`
- `grooming::event::CrossSellCustomerDraftPrepared`
- `grooming::event::CrossSellOfferApprovedForSend`
- `grooming::event::CrossSellOfferSent`
- `grooming::event::CrossSellAccepted`
- `grooming::event::CrossSellDeclined`
- `grooming::event::ReservationAddOnDraftCreated`
- `grooming::event::AppointmentRequestDrafted`
- `grooming::event::CrossSellExecutionRejectedByPolicy`

### Staff tasks

- `operations::StaffTaskKind::CustomerFollowUp { reason: FollowUpReason::PostStayCheckIn }` can be reused for generic reviewed follow-up.
- Add `GroomingCrossSellReview { opportunity_id }` when a generic customer follow-up loses too much meaning.
- Add `GroomerCalendarReview { opportunity_id, pet_id }` when calendar/duration suitability is specifically groomer-owned.
- Add `DocumentReview { pet_id }` or future care-review task when medical/care facts block safe recommendation.
- Manager tasks handle discount/deposit/no-show/calendar overrides.

### Agent specs/tools

- `grooming::agent::CrossSellSpec` consumes domain packet values: source context, menu, history summary, care-risk flags, calendar summary, consent summary, and prior outcomes.
- `grooming::agent::CrossSellDraft` returns recommendation, evidence, confidence, draft copy, and risk flags.
- `CrossSellPolicy::evaluate_agent_draft` promotes or rejects the draft into a `CrossSellDecision`.
- Tools accept only approved commands with typed opportunity IDs and approval tokens. A tool should not accept raw LLM text such as "add exit bath" as a command.

## 4. Interaction contract

Rust-like pseudo-signatures below name ownership, not final compile-ready code.

```rust
impl grooming::CrossSellPolicy {
    pub fn evaluate(
        &self,
        source: cross_sell::SourceServiceContext,
        reservation: &reservation::Summary,
        customer: &customer::ContactPolicySnapshot,
        pet: &pet::ProfileSnapshot,
        care: &care::ReviewSnapshot,
        menu: &grooming::ServiceMenu,
        history: &grooming::ServiceHistory,
        contract: &grooming::Contract,
        calendar: Option<&grooming::CalendarSnapshot>,
        prior_outcomes: &[cross_sell::PriorOutcome],
        today: calendar::ServiceDate,
    ) -> grooming::Result<grooming::CrossSellDecision>;
}
```

Behavior:

- Returns `Suppressed` when the opportunity is duplicate, unsupported, unsafe, or impossible.
- Returns `NeedsReview` when staff/groomer/manager/care review is required before even drafting customer copy.
- Returns `Opportunity` only when there is a safe internal next action with explicit automation boundary.
- Never writes to reservation, calendar, payment, or messaging systems.

```rust
impl grooming::ServiceMenu {
    pub fn allows_cross_sell(
        &self,
        offering: grooming::ServiceOffering,
        source: &cross_sell::SourceServiceContext,
    ) -> cross_sell::MenuDecision;
}
```

Behavior belongs on `ServiceMenu` because it owns location-enabled services and source-specific availability.

```rust
impl grooming::HistoryRepository {
    pub async fn service_history_for(
        &self,
        pet_id: entities::PetId,
        location_id: entities::LocationId,
    ) -> grooming::Result<grooming::ServiceHistory>;

    pub async fn prior_cross_sell_outcomes(
        &self,
        pet_id: entities::PetId,
        customer_id: entities::CustomerId,
    ) -> grooming::Result<Vec<cross_sell::PriorOutcome>>;
}
```

History owns service/cadence evidence. The cross-sell policy should not scrape raw notes.

```rust
impl grooming::EstimationPolicy {
    pub fn estimate_for_cross_sell(
        &self,
        offering: grooming::ServiceOffering,
        pet: &pet::ProfileSnapshot,
        care: &care::ReviewSnapshot,
        history: &grooming::ServiceHistory,
        contract: &grooming::Contract,
    ) -> grooming::Result<grooming::DurationEstimate>;
}
```

Estimation owns duration/review implications. `CrossSellPolicy` asks it for appointment-shaped offers rather than embedding breed/coat logic.

```rust
impl grooming::SchedulingPolicy {
    pub fn same_day_checkout_candidate(
        &self,
        source: &cross_sell::SourceServiceContext,
        estimate: &grooming::DurationEstimate,
        calendar: &grooming::CalendarSnapshot,
        contract: &grooming::Contract,
    ) -> grooming::CalendarDecision;
}
```

Scheduling owns calendar fit. It may return `NoCandidate` without suppressing all follow-up; a future appointment draft may still be valid.

```rust
impl grooming::AutomationPolicy {
    pub fn decide_cross_sell_action(
        &self,
        opportunity: &grooming::CrossSellOpportunity,
        requested: cross_sell::RequestedAction,
        actor: policy::Actor,
        approval: Option<&grooming::ApprovalToken>,
    ) -> policy::AutomationDecision;
}
```

Automation owns whether a requested action is internal-only, approval-required, executable, or disallowed. It must reject booking, payment, add-on mutation, and member-facing messages from an AI recommendation alone.

```rust
impl grooming::CrossSellRepository {
    pub async fn save_decision(
        &self,
        decision: &grooming::CrossSellDecision,
        audit: policy::AuditContext,
    ) -> grooming::Result<()>;

    pub async fn record_customer_response(
        &self,
        opportunity_id: cross_sell::OpportunityId,
        response: cross_sell::CustomerResponse,
        audit: policy::AuditContext,
    ) -> grooming::Result<()>;
}
```

Repository owns durable opportunity lifecycle. Customer response does not directly schedule or charge; it becomes input to reservation/grooming workflows.

```rust
impl grooming::agent::CrossSellSpec {
    pub fn from_domain_packet(packet: grooming::CrossSellPromptPacket) -> Self;
}

impl grooming::CrossSellPolicy {
    pub fn evaluate_agent_draft(
        &self,
        draft: grooming::agent::CrossSellDraft,
        deterministic_context: &grooming::CrossSellPolicyContext,
    ) -> grooming::CrossSellDecision;
}
```

The agent can propose; deterministic policy promotes, narrows, or rejects.

## 5. Review and approval contract

### Automation level

Allowed without human approval:

- Internal ranking of candidates.
- Internal suppression of unsafe/duplicate opportunities, with audit reason.
- Creation of an internal recommendation record.
- Drafting suggested staff-task text or customer-message copy, as a draft only.
- Summarizing evidence and conflict reasons for staff review.

Requires front desk/staff approval:

- Any customer-facing offer/message.
- Any reservation add-on draft presented to a customer.
- Any generic post-stay/daycare follow-up that mentions pricing, availability, or time windows.
- First-time offer wording if eligibility is mechanically clear but still member-facing.

Requires groomer review:

- Full groom, premium bath, matted coat, sensitive skin, style interpretation, unknown breed/coat data, product-specific recommendations, or duration estimate uncertainty.
- Same-day checkout scheduling when groomer capacity or quality would be affected.

Requires manager review:

- Calendar override, overbooking, buffer reduction, discount/first-time-offer ambiguity, deposit/no-show restriction, refund/complaint context, repeated declined offers, or care/safety ambiguity.

Disallowed from automation alone:

- Sending the customer message.
- Adding a grooming service/product to a reservation or invoice.
- Booking/rescheduling/cancelling a grooming appointment.
- Charging, waiving, discounting, or applying deposits.
- Making medical suitability claims.

### Review gates

1. Deterministic eligibility gate: source reservation, menu availability, duplicate detection, and customer consent status are checked before a customer-facing draft is allowed.
2. Care/safety gate: care facts, medical products, sensitive skin, matted coat, incident history, or handling ambiguity force staff/groomer/care review.
3. Operational capacity gate: same-day or appointment-shaped offers require duration/calendar feasibility or are converted to future follow-up.
4. Commercial/payment gate: price, deposit, discount, no-show, and first-time offer rules produce staff/manager review; payment systems remain out of scope.
5. Member-facing gate: any send requires consent plus human approval or an explicit pre-approved campaign policy.
6. Tool-execution gate: reservation add-on, appointment hold, booking, payment, or message tools require typed approval tokens scoped to the action.

### Audit trail

Persist:

- source trigger, reservation, customer, pet, location, and time;
- policy version and contract/menu snapshot versions;
- recommended offering and alternatives considered;
- evidence, suppression/review reasons, and AI draft ID if present;
- automation decision and approval boundary;
- reviewer identity/role, approval token, timestamp, and scoped action;
- customer response and final outcome;
- downstream tool command IDs and success/failure if execution occurs.

### Customer/member-facing boundaries

- The customer should see a clear optional offer, not a hidden add-on.
- Offer copy must avoid implying the service is required for checkout/daycare unless policy truly says so.
- The message must not disclose internal risk labels or sensitive care notes.
- Acceptance should be explicit and separately auditable before booking/charging.
- Declines should be recorded to avoid spammy repeated offers.

## 6. Test contracts

Domain tests should read as an executable glossary. Suggested names:

1. `boarding_checkout_exit_bath_cross_sell_creates_opportunity_without_mutating_reservation`
   - Given a boarding checkout source, enabled `ExitBath`, consent available, and no duplicate add-on, policy returns `Opportunity` with `DraftCustomerMessage` or `CreateStaffTask`; reservation state remains unchanged.
2. `daycare_visit_cross_sell_uses_daycare_source_context_not_generic_grooming_lead`
   - A daycare source produces `SourceServiceContext::DaycareVisit` and preserves visit/reservation identity.
3. `cross_sell_policy_suppresses_offer_when_equivalent_grooming_already_selected`
   - Duplicate selected add-ons or scheduled grooming return `Suppressed(OfferAlreadySelected)`.
4. `cross_sell_policy_rejects_menu_disabled_offering`
   - Disabled location service returns `Suppressed(MenuOfferingUnavailable)` instead of a draft.
5. `exit_bath_after_boarding_can_be_recommended_without_full_groom_duration_estimate`
   - Exit bath recommendation may use simpler contract/menu evidence, while full groom requires duration/calendar review.
6. `full_groom_cross_sell_requires_duration_estimate_and_groomer_review_when_breed_or_coat_unknown`
   - Unknown or risky breed/coat data produces `GroomerReview`/`CareOrMedicalReview`.
7. `matted_or_sensitive_coat_blocks_member_facing_cross_sell_until_staff_review`
   - Care-sensitive facts prevent direct customer message approval.
8. `no_show_history_requires_deposit_or_manager_review_before_rebooking_offer`
   - No-show/late-cancel history maps to deposit/manager boundary, not a silent offer.
9. `cross_sell_customer_message_requires_consent_and_approval_token_before_send`
   - Draft can exist; send execution is rejected without consent/approval.
10. `cross_sell_agent_draft_is_never_tool_execution_permission`
    - AI draft alone maps to `StaffApprovalRequired` or `Disallowed` for booking/payment/send/add-on mutation.
11. `cross_sell_repository_records_suppressed_opportunity_with_typed_reason`
    - Suppression persists an auditable reason and source context.
12. `customer_decline_suppresses_repeated_same_offer_for_policy_window`
    - Prior decline affects future opportunity ranking/suppression.
13. `calendar_unavailable_converts_same_day_checkout_offer_to_future_follow_up_task`
    - No same-day slot does not fabricate availability; it creates a staff task or future appointment draft.
14. `first_time_grooming_offer_requires_offer_repository_eligibility_or_manager_review`
    - Promotional ambiguity is not represented as a boolean.
15. `cross_sell_audit_trail_links_trigger_policy_review_customer_response_and_tool_outcome`
    - Lifecycle audit has all required typed references.

Storage/boundary tests:

1. `cross_sell_record_roundtrips_source_context_offering_evidence_review_and_boundary`
2. `cross_sell_record_rejects_raw_provider_id_without_semantic_promotion`
3. `reservation_add_on_draft_does_not_serialize_as_committed_add_on`
4. `agent_cross_sell_draft_promotes_only_through_deterministic_policy`
5. `message_tool_rejects_cross_sell_send_without_scoped_approval_token`

## 7. Integration notes for later serialized Rust code card

### Files likely touched

Domain core:

- `domain/src/operations.rs`
  - Add `operations::grooming::cross_sell` submodule or split grooming into `domain/src/operations/grooming/*.rs` if the file becomes too large.
  - Add `CrossSellOpportunity`, source context, decision, evidence, suppression, review/automation types, and policy contracts.
  - Add/extend grooming-owned `ServiceOffering` and adapter from current top-level `operations::GroomingService`.
  - Add explicit staff task variants only if generic task kinds are insufficient.
- `domain/src/reservation/mod.rs`
  - Add or reference typed summary/add-on draft concepts if reservation code owns committed add-ons.
- `domain/src/customer.rs`, `domain/src/pet.rs`, `domain/src/care.rs`
  - Expose snapshot/value contracts needed by the policy without leaking raw records.
- `domain/src/workflow.rs`, `domain/src/agent.rs`, `domain/src/agents.rs`, `domain/src/tools.rs`
  - Add typed event/action/tool command contracts for reviewed draft messages/tasks/add-on drafts/appointment holds if not already available.

Tests:

- `domain/tests/petsuites_core_service_contracts.rs`
  - Keep existing grooming contract coverage.
- New `domain/tests/grooming_cross_sell_contracts.rs`
  - Add semantic contract tests listed above.
- Storage tests under the storage crate if cross-sell records are serialized later.
- Agent/tool boundary tests when an AI or provider adapter card implements execution.

Docs:

- `docs/domain/petsuites/grooming/service-domain-map.md`
  - May link this implication once implication index/navigation exists.
- `docs/domain/petsuites/grooming/implications/05-cross-sell-grooming-after-daycare-boarding.md`
  - This artifact is the modeling authority for the serialized Rust card.

### Migration/refactor risks

- Current `operations::RevenueOpportunityKind::ExitBathAfterBoarding` is useful but too narrow for daycare, first-time, and lapsed-cadence grooming. Do not replace it with a vague string; add precise variants or a grooming-owned opportunity entity.
- Current top-level `operations::GroomingService` can bootstrap storage compatibility, but behavior should move toward `operations::grooming::ServiceOffering` when grooming-specific policy grows.
- Avoid duplicate cadence types (`operations::CadenceWeeks` and `operations::grooming::CadenceWeeks`) gaining divergent semantics. Grooming ordinary cadence should enforce/express the 2-8 week norm or an explicit override.
- Do not model `accepted: bool`, `sent: bool`, `approved: bool`, or `added_to_reservation: bool`; use state/decision enums with actor, timestamp, and boundary proof.
- Do not let storage/provider records become the domain API. Promote raw provider reservation IDs, menu codes, and message IDs at adapter boundaries.
- Avoid free-floating helper functions such as `is_cross_sell_candidate`; put behavior on `CrossSellPolicy`, `ServiceMenu`, `HistoryRepository`, `EstimationPolicy`, `SchedulingPolicy`, or `AutomationPolicy`.
- Cross-sell can become spammy if prior decline/outcome is ignored. Model prior outcome and suppression windows early.
- Member-facing copy must not expose sensitive care/medical details or imply services are mandatory.

### Dependencies on other implications/domain work

- Boarding checkout/add-on modeling: source reservation and checkout windows should be typed before same-day exit bath execution.
- Daycare visit/package modeling: daycare source context should distinguish visit completion from generic lead conversion.
- Grooming rebooking/cadence implication: lapsed cadence and next-due recommendations feed this cross-sell path.
- Grooming calendar/duration implication: full groom and same-day appointment-shaped offers depend on duration and calendar policies.
- Grooming history/notes implication: prior service, style notes, and product history feed evidence and review gates.
- Customer communication/consent: no member-facing send should bypass channel consent and approval.
- Payment/reservation add-on boundaries: accepted offers still require reservation/payment domain actions, not grooming-side mutation.

### Implementation shape for later Rust card

Start with tests, then types, then policies:

1. Add `grooming_cross_sell_contracts.rs` with RED tests for recommendation-not-mutation, duplicate suppression, consent/approval gate, and AI-not-execution-permission.
2. Add semantic types in `operations::grooming::cross_sell` with constructors/builders enforcing non-empty evidence and exact source context.
3. Add `CrossSellPolicy::evaluate` using deterministic snapshots and returning `CrossSellDecision`.
4. Add adapter from `operations::GroomingService` to grooming-owned `ServiceOffering` if the canonical move cannot happen in the same card.
5. Add `AutomationPolicy::decide_cross_sell_action` or extend existing policy surfaces to reject unsafe execution.
6. Add repository traits only after the domain tests prove the contract; storage implementation can follow in a separate card.
7. Keep AI agent specs draft-only until deterministic policy and approval-token tests exist.

The central invariant for implementation: cross-sell is a reviewed opportunity lifecycle. It is not a shortcut around reservation, calendar, payment, care, consent, or human approval boundaries.
