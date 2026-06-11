# PetSuites grooming service domain map

Purpose: define the semantic domain surface for PetSuites grooming inside the NVA/PetSuites AI-operations foundation. This is a contract map for later Rust/domain cards, not an implementation patch. It preserves domain meaning in module paths and type names so scheduling, rebooking, reminders, history, and AI-agent recommendations do not collapse into raw strings, booleans, or helper-shaped workflow code.

Assumptions:

- PetSuites grooming is a core service line but often attaches to boarding/daycare reservations as an add-on or cross-sell.
- Location-level service menus, prices, groomer availability, breed restrictions, deposit/no-show rules, and first-time offers may vary by resort.
- AI agents may draft, rank, summarize, and recommend. Deterministic Rust policies and human approval gates decide whether bookings, payments, staff schedules, or customer messages are executed.
- This map intentionally models extensible contract shapes; provider-specific identifiers and raw Gingr/POS/calendar payloads remain in storage/integration adapters.

## 1. Domain vocabulary and bounded context

Bounded context: `operations::grooming` owns the operational grooming contract for a resort location: service menu, appointment duration estimation, groomer calendar rules, no-show/cancellation handling, rebooking cadence, reminders, service history, and safe AI recommendations. It should not own customer identity, pet medical/care facts, payment capture, reservation lifecycle, or messaging delivery. It consumes those neighboring domain values through typed IDs, policies, repositories, and draft/action contracts.

Core vocabulary:

- Grooming service line: the location-specific operational contract for bathing, grooming, nail, ear, coat/skin product, and promotional services.
- Service offering: an item the customer can request or staff can add to a reservation, e.g. mini groom, full groom, exit bath, full bath, premium bath, nail trim, nail Dremel, ear cleaning, coat/skin-specific product, first-time grooming offer.
- Grooming appointment: a scheduled or proposed block on a groomer calendar for one pet and one service plan.
- Groomer calendar: staff-capacity surface for named/qualified groomers, appointment blocks, buffers, and manager overrides.
- Breed/coat time estimate: predicted or policy-set duration derived from breed group, species, coat condition, size, matting, service type, and historical groomer notes.
- Service plan: the accepted bundle of offering, options, products, duration, groomer assignment, reservation/add-on linkage, deposit/price quote, and approval state.
- Rebooking cadence: the recommended interval for the next appointment, commonly every 2-8 weeks but overridable by groomer recommendation or care need.
- No-show/cancellation management: policy surface for late cancellation, no-show history, deposit requirement, rebooking hold, and manager review.
- Groomer notes/service history: style instructions, coat/skin observations, behavior/handling notes, photos/media references, customer preferences, and outcomes from prior appointments.
- Cross-sell opportunity: a recommendation to offer grooming after daycare/boarding or before checkout, without silently adding charges or changing reservations.
- Reminder: drafted or scheduled customer communication for upcoming grooming appointment, rebooking due, preparation instructions, or confirmation request.

Out of scope for this context:

- `customer`: identity, preferences, communication consent, household/account state.
- `pet`/`care`: pet identity, species/breed facts, medical/handling facts, allergies, vaccinations, care profile.
- `reservation`: booking lifecycle, boarding/daycare/training reservation status, add-on status.
- `money`/`payment`: price amounts, deposits, invoices, refunds, payment authorization/capture.
- `workflow`/`tools`: external provider IO, message delivery, task creation, audit logging.

## 2. Domain type inventory using semantic paths

Current flat surface to reuse short term:

- `operations::ServiceOffering::Grooming { service, cadence }`
- `operations::GroomingService`
- `operations::GroomingCadence`
- `operations::CadenceWeeks`
- `operations::grooming::Contract`
- `operations::grooming::CalendarPolicy`
- `operations::grooming::BreedCategory`
- `operations::grooming::CoatCondition`
- `operations::grooming::BreedCoatTimeEstimate`
- `operations::grooming::AppointmentMinutes`
- `operations::grooming::NoShowPolicy`
- `operations::grooming::RebookingCadence`
- `operations::grooming::ReminderRule`
- `operations::grooming::HistoryRequirement`

Recommended canonical surface after the next code slice:

- `operations::grooming::Contract`
  - Location-level contract for calendar, service menu, estimation, no-show/cancellation, rebooking, reminders, history, and automation boundaries.
- `operations::grooming::ServiceOffering`
  - Grooming-owned offering enum. Prefer this over the current cross-service `operations::GroomingService` when behavior becomes grooming-specific.
  - Variants: `MiniGroom`, `FullGroom`, `ExitBath`, `FullBath`, `PremiumBath`, `NailTrim`, `NailDremel`, `EarCleaning`, `CoatSkinSpecificProduct`, `FirstTimeOffer`.
- `operations::grooming::ServiceMenu`
  - Non-empty set of location-enabled offerings with optional availability/eligibility rules.
- `operations::grooming::AppointmentRequest`
  - Draft request from customer/staff/agent before scheduling. Carries typed customer/pet/location IDs, desired offering, source context, and optional reservation link.
- `operations::grooming::AppointmentPlan`
  - Validated proposed appointment with duration estimate, groomer assignment policy, deposit/price quote references, required approvals, and reminder plan.
- `operations::grooming::AppointmentId`
  - Provider/domain ID for a grooming appointment once persisted. Provider IDs must be converted at the boundary.
- `operations::grooming::DurationEstimate`
  - `AppointmentMinutes` plus `EstimateBasis`, confidence, and any required staff review.
- `operations::grooming::EstimateBasis`
  - `BreedCoatPolicy`, `GroomerHistory`, `ProviderDefault`, `ManualStaffOverride`, `AiSuggestedPendingReview`.
- `operations::grooming::BreedCategory`
  - Keep current categories, but expect expansion to `SmallShortCoat`, `LargeDoubleCoat`, `DoodleOrPoodleMix`, `Cat`, `UnknownRequiresReview` if estimates branch by size/species.
- `operations::grooming::CoatCondition`
  - Extend current `Maintained`, `ThickUndercoat`, `Matted` with `SensitiveSkin`, `FleaTickConcern`, `PostBoardingOdor`, `UnknownRequiresReview` if policy needs them.
- `operations::grooming::ProductNeed`
  - Coat/skin-specific products as typed needs rather than text add-ons: `SensitiveSkinShampoo`, `Deshedding`, `MedicatedProductRequiresStaffReview`, `FleaTickProductRequiresPolicyReview`, `Other(extension::Label)`.
- `operations::grooming::CalendarPolicy`
  - Existing owner for `AnyQualifiedGroomer`, `GroomerSpecific`, `FirstAvailableWithManagerOverride`.
- `operations::grooming::GroomerAssignment`
  - `AnyQualified`, `Preferred(staff::Id)`, `Required(staff::Id)`, `ManagerOverride(staff::Id)`.
- `operations::grooming::ScheduleWindow`
  - Desired/proposed appointment time range; do not pass naked `DateTime` pairs around scheduling policies.
- `operations::grooming::CalendarBlock`
  - Existing appointment, hold, buffer, lunch/blackout, or staff-unavailable block.
- `operations::grooming::BufferMinutes`
  - Validated non-zero or explicit zero-allowed newtype depending on local policy.
- `operations::grooming::NoShowPolicy`
  - Existing enum; add cancellation-specific vocabulary if late cancellations differ from no-shows.
- `operations::grooming::CancellationPolicy`
  - `NoPenaltyUntil`, `RequireDepositForRebooking`, `ManagerReviewAfterRepeatedLateCancel`, `ReleaseSlotToWaitlist`.
- `operations::grooming::RebookingCadence`
  - Existing enum; constrain ordinary recurring cadence with a `CadenceWeeks` policy of 2-8 weeks.
- `operations::grooming::CadenceWeeks`
  - Grooming-specific cadence value. The current code has both `operations::CadenceWeeks` and `operations::grooming::CadenceWeeks`; future code should choose one canonical owner and avoid duplicate semantics.
- `operations::grooming::RebookingRecommendation`
  - Pet/customer-specific recommendation with due window, rationale, confidence, source, and approval requirement.
- `operations::grooming::ReminderRule`
  - Existing enum; future variants should model channel/content policy separately from timing.
- `operations::grooming::ReminderPlan`
  - Sequence of customer-message drafts or staff tasks tied to consent and approval decisions.
- `operations::grooming::HistoryRequirement`
  - Existing enum; expand through typed service-history entities rather than free-form text flags.
- `operations::grooming::ServiceHistoryEntry`
  - Completed-service record with offering, date, groomer, outcome, notes, product use, duration, photos/media references, and next-cadence recommendation.
- `operations::grooming::StyleNote`
  - Validated note for cut/style/customer preference; separate from care/medical handling notes.
- `operations::grooming::HandlingNoteRef`
  - Reference to care/temperament notes; grooming should not duplicate sensitive medical/behavior facts.
- `operations::grooming::CrossSellOpportunity`
  - Recommendation after daycare/boarding or pre-checkout, with source reservation, offering, timing, and review/send boundary.
- `operations::grooming::ApprovalBoundary`
  - Domain-facing automation gate for schedule, payment, customer communication, and pet-care-sensitive decisions.

Repository/domain service paths:

- `operations::grooming::Repository`
  - Loads contract, menu, appointment, history, and location policy snapshots in domain form.
- `operations::grooming::CalendarRepository`
  - Reads groomer availability and existing blocks; writes only draft holds unless an approval policy permits booking.
- `operations::grooming::HistoryRepository`
  - Reads/writes service history through semantic entries, not raw note strings.
- `operations::grooming::EstimationPolicy`
  - Calculates duration and review requirement from service, breed/coat, pet profile, and history.
- `operations::grooming::SchedulingPolicy`
  - Chooses viable appointment windows and explains conflicts.
- `operations::grooming::RebookingPolicy`
  - Converts service history and cadence into due/overdue recommendations.
- `operations::grooming::NoShowPolicyEngine`
  - Applies no-show/cancellation history to deposits, holds, and review requirements.
- `operations::grooming::CrossSellPolicy`
  - Identifies safe grooming offers from boarding/daycare context.
- `operations::grooming::AutomationPolicy`
  - Maps proposed action to draft-only, staff approval, manager approval, or disallowed.

## 3. Existing Rust/domain surface to reuse or refactor

Existing `domain/src/operations.rs` provides a useful first contract:

- `operations::ServiceOffering::Grooming { service: GroomingService, cadence: GroomingCadence }` already preserves cross-service variant shape and is tested through `storage/tests/operations_storage_contracts.rs`.
- `operations::GroomingService` covers the required source surface: mini groom, full groom, exit bath, full bath, premium bath, nail trim, nail Dremel, ear cleaning, coat/skin-specific product, and first-time offer.
- `operations::GroomingCadence` plus top-level `operations::CadenceWeeks` supports current rebooking cadence, but it only enforces non-zero weeks. Later grooming-specific behavior should enforce the normal 2-8 week band through `operations::grooming::RebookingCadence`/`operations::grooming::CadenceWeeks` or a named policy.
- `operations::grooming::Contract::standard_petsuites()` already models the location-level contract with groomer-specific calendar policy, doodle/matted 180-minute estimate, no-show deposit requirement, six-week rebooking, 48-hour/morning-of reminders, and style/photo history.
- `operations::grooming::BreedCoatTimeEstimate` already places appointment minutes behind `AppointmentMinutes`, which is better than raw durations.
- `operations::CoreServiceContracts` includes `grooming::Contract` and is round-tripped in `storage/tests/core_service_contract_storage.rs`.
- `OperationsAction`, `RevenueOpportunityKind::GroomingRebookingDue`, `ReviewTheme::GroomingOutcome`, `AiUseCase::GroomingRebooking`, `OperatingFunction::Grooming`, and `CapacityConstraintKind::GroomerSlotAvailability` are reusable integration points for daily briefs, reputation, workflow recommendations, and operational reports.

Refactor opportunities when behavior grows:

1. Move grooming-specific service/cadence types from the top-level cross-service enum into `operations::grooming`, then re-export only the canonical public surface needed by call sites.
2. Avoid having both `operations::CadenceWeeks` and `operations::grooming::CadenceWeeks` represent the same grooming cadence. If a top-level cadence remains, make it generic and never attach grooming policy such as 2-8 week limits to it.
3. Split service menu (`ServiceOffering`/`ServiceMenu`) from operational contract (`Contract`). `Contract` governs policy; menu entries represent what a customer can request at a location.
4. Promote duration confidence/review requirements into `DurationEstimate`; do not infer staff review from a magic number or missing estimate.
5. Expand `StaffTaskKind` with grooming-specific tasks only when workflow needs them, e.g. `GroomingAppointmentPrep`, `GroomingHistoryReview`, `GroomingRebookingFollowUp`. Until then, use `CustomerFollowUp` or `DocumentReview` only if the semantics are truthful.
6. Keep storage records (`storage::operations::*Record`) as boundary shapes. Convert raw provider codes and optional columns into semantic variants before domain behavior sees them.

## 4. Required newtypes, enums, builders, policies, repositories, services

Newtypes and scalar contracts:

- `operations::grooming::AppointmentId`: non-empty provider/domain ID or UUID wrapper, depending on persistence strategy.
- `operations::grooming::AppointmentMinutes`: already exists; continue requiring at least one minute.
- `operations::grooming::BufferMinutes`: explicit buffer duration, with policy deciding whether zero is legal.
- `operations::grooming::CadenceWeeks`: grooming cadence weeks; ordinary recurring recommendations should be 2-8 weeks unless an explicit `GroomerRecommended` or manager override variant explains the exception.
- `operations::grooming::OfferCode`: validated location/provider offer code for first-time or promotional offers.
- `operations::grooming::StyleNote`: trimmed, bounded text; may be sensitive and should have redacted debug behavior if it can contain customer/pet specifics.
- `operations::grooming::ProductInstruction`: validated product-use instruction; medical products should require staff/manager review.
- `operations::grooming::ConfidenceBasisPoints`: bounded 0-10,000 if AI or statistical estimates are represented numerically.
- `operations::grooming::CancellationCount` / `NoShowCount`: bounded counts for policy decisions.

Enums:

- `operations::grooming::ServiceOffering`: required source service surface as grooming-owned variants.
- `operations::grooming::ServiceCategory`: `Bath`, `Haircut`, `NailCare`, `EarCare`, `CoatSkinProduct`, `Promotion` for reporting/menu grouping only; do not use category when exact offering matters.
- `operations::grooming::AppointmentState`: `Draft`, `HoldPendingApproval`, `Scheduled`, `Completed`, `Cancelled`, `NoShow`, `NeedsManagerReview` if grooming appointments gain lifecycle behavior separate from reservation status.
- `operations::grooming::EstimateBasis`: as above.
- `operations::grooming::ReviewRequirement`: `None`, `StaffReview`, `GroomerReview`, `ManagerReview`, `CareOrMedicalReview`.
- `operations::grooming::CrossSellSource`: `BoardingCheckout`, `DaycareVisit`, `LapsedGroomingCadence`, `FirstTimeCustomer`, `StaffSuggested`.
- `operations::grooming::ReminderKind`: `AppointmentConfirmation`, `PrepInstructions`, `MorningOf`, `RebookingDue`, `LapsedCadenceWinback`.
- `operations::grooming::AutomationBoundary`: `DraftOnly`, `StaffApprovalRequired`, `ManagerApprovalRequired`, `MemberFacingSendRequiresApproval`, `Disallowed`.

Builders:

- `Contract::builder()` already exists and should remain the location-level construction surface.
- `ServiceMenu::builder()` should require a non-empty enabled offering set and reject duplicate offering entries.
- `AppointmentRequest::builder()` should require `location_id`, `customer_id`, `pet_id`, `offering`, and `source`; reservation link, preferred groomer, requested window, and notes are optional typed fields.
- `AppointmentPlan::builder()` should require request, duration estimate, calendar decision, approval boundary, and reminder policy before it can become schedulable.
- `ServiceHistoryEntry::builder()` should require completed appointment/service, date, pet, location, offering, and outcome. Sensitive notes should be typed optional fields with redacted debug.
- If phase-specific legality matters later, replace runtime `AppointmentState` mutation with a typestate builder for `DraftAppointment -> ApprovedAppointment -> ScheduledAppointment -> CompletedAppointment`.

Policies/domain services:

- `EstimationPolicy::estimate(request, pet_profile, service_history, contract) -> DurationEstimate`
  - Invariants: duration must be positive; low confidence or missing breed/coat data produces review requirement; matted/sensitive/medical-product cases cannot be silently auto-scheduled.
- `SchedulingPolicy::rank_windows(request, estimate, calendar, contract) -> Vec<ScheduleCandidate>`
  - Invariants: no overlap with groomer blocks; required groomer qualification/preference is honored; manager override is represented explicitly; buffers are included.
- `NoShowPolicyEngine::evaluate(customer_id, pet_id, history, contract) -> NoShowDecision`
  - Invariants: repeated no-shows/late cancels produce deposit or manager review; agents cannot waive penalties.
- `RebookingPolicy::recommend(history, contract, today) -> RebookingRecommendation`
  - Invariants: ordinary cadence stays in 2-8 weeks; overdue recommendations carry due date and rationale; customer communication is draft until consent/approval passes.
- `CrossSellPolicy::evaluate(reservation, pet_profile, customer_history, contract) -> Option<CrossSellOpportunity>`
  - Invariants: do not recommend incompatible services; do not add charges or modify reservations; distinguish exit bath after boarding from full groom/new appointment.
- `ReminderPolicy::plan(appointment, customer_preferences, consent) -> ReminderPlan`
  - Invariants: no member-facing send without communication consent and approval boundary; reminders carry truthful timing/kind.
- `HistoryPolicy::record(completed_appointment, groomer_notes) -> ServiceHistoryEntry`
  - Invariants: style notes are separate from care/medical/temperament notes; medical-sensitive content is referenced or gated, not copied into generic notes.
- `AutomationPolicy::decide(action, risk, confidence, policy_context) -> AutomationDecision`
  - Invariants: booking, payment/deposit, cancellation penalty, manager override, and member-facing messages are never executed solely from an LLM suggestion.

Repositories:

- `operations::grooming::Repository`: read/write aggregate-level grooming appointment/menu/history data in domain form.
- `operations::grooming::ContractRepository`: load location/brand contract snapshots.
- `operations::grooming::CalendarRepository`: read calendar availability and create draft holds; final booking goes through a tool/policy port.
- `operations::grooming::HistoryRepository`: read service history for estimates/rebooking and append completed-service entries.
- `operations::grooming::OfferRepository`: load enabled location offers and promotional/first-time offer rules.

## 5. Relationships to neighboring modules

- `customer`: use `entities::CustomerId` and customer communication preferences/consent. Grooming does not own account identity or permission to contact. Reminder and rebooking drafts must consult customer consent and channel policy.
- `pet`: use `entities::PetId`, species/breed/size values, and pet profile facts. Breed/coat facts feed estimation, but grooming should not invent pet identity fields.
- `care`: care profile owns allergies, medical conditions, handling requirements, vaccination or health-document facts. Grooming may reference care review requirements for sensitive products, matted/painful handling, or medical shampoo, but care/medical decisions stay in `care`/`policy`.
- `reservation`: boarding/daycare reservations can source exit-bath or grooming cross-sell opportunities. Grooming appointment plans should link through typed `ReservationId`/add-on references rather than mutate reservation status directly.
- `location`: location owns resort identity, local service availability, hours, and staff availability context. `operations::grooming::Contract` should be loaded per location.
- `staff task`: grooming can produce staff tasks for history review, customer follow-up, manager review, document/care review, or groomer assignment. Use explicit `StaffRole::Groomer`/`StaffId`; add grooming-specific `StaffTaskKind` variants only when existing kinds are semantically insufficient.
- `money`/`payment`: deposits, prices, package credits, refunds, waivers, and invoices belong to `money`/`payment`. Grooming policies can return `DepositRequired`/`PriceQuoteRequired` decisions using money types, but cannot capture or waive payment itself.
- `workflow`: AI/workflow modules may produce `RecommendedAction`, `DraftCustomerMessage`, `CreateInternalTask`, or `SuggestScheduleReview`. Deterministic policy gates validate action legality before tools execute.
- `agents`: grooming agent specs should receive typed prompt packets and return typed recommendations with risk flags, confidence, review reasons, and evidence. No raw free-form command should directly book, charge, cancel, or message a customer.
- `tools`: provider integrations (Gingr calendar, POS, messaging, document/media, Hermes task creation) are boundary ports. Domain services output draft commands or decisions; tools execute only after approval policies pass.
- `storage`: storage records preserve provider payload shape and codec compatibility. Storage must reject cross-variant shapes, as current service-offering storage tests already do.

## 6. AI-agent opportunities and approval boundaries

Safe to automate as draft/recommendation:

- Rank available grooming windows by groomer fit, duration estimate, buffers, and reservation checkout context.
- Predict appointment duration from breed/coat/service/history and explain the basis.
- Detect rebooking-due customers/pets from service history and cadence.
- Identify cross-sell candidates after daycare/boarding, especially exit bath before checkout.
- Draft reminder, confirmation, rebooking, or prep-instruction messages.
- Summarize groomer notes into a structured history draft for staff review.
- Flag missing breed/coat data, inconsistent service naming, overdue rebooking, repeated no-shows, or calendar bottlenecks.
- Draft internal tasks for staff/groomer/manager review.

Requires staff/groomer review:

- Low-confidence time estimates or missing breed/coat/size data.
- Matted coat, sensitive skin, behavior/handling concerns, or medical-product needs.
- Style-note interpretation that could change the haircut/grooming outcome.
- First-time grooming offer eligibility when location/provider rules are ambiguous.
- Cross-sell recommendations that depend on care/medical/temperament facts.
- Any generated service-history note before it becomes durable staff record.

Requires manager review:

- Overriding groomer-specific calendar policy or double-booking/buffer conflicts.
- Rebooking after repeated no-shows or late cancellations when policy imposes deposit/hold restrictions.
- Waiving deposits, discounts, cancellation penalties, or first-time offer restrictions.
- Customer complaint/reputation escalation involving grooming outcome, pet injury/safety, or refund request.
- Any action with legal/safety implications or unresolved care/medical ambiguity.

Member-facing or unsafe without approval:

- Sending appointment confirmations, reminders, rebooking offers, price/deposit demands, or apologies directly to customers.
- Booking/rescheduling/cancelling appointments in provider systems.
- Charging, refunding, waiving, or applying deposits/discounts.
- Adding grooming services or products to reservations/invoices.
- Telling a customer a pet is medically suitable for a product or service.
- Hiding or rewriting concerning groomer notes, incident details, injuries, or complaint facts.

Approval-boundary contract shape:

```rust
enum AutomationDecision {
    DraftOnly { reason: policy::AutomationRationale },
    StaffApprovalRequired { role: operations::StaffRole, reason: policy::ReviewReason },
    ManagerApprovalRequired { reason: policy::ReviewReason },
    ApprovedForToolExecution { approval: policy::ApprovalToken },
    Disallowed { reason: policy::DenialReason },
}
```

The exact names can change, but the invariant should not: an LLM recommendation is not an execution permission.

## 7. Acceptance tests/contracts for later code cards

Suggested tests should live in focused domain/storage test files and read like executable glossary entries.

Domain contract tests:

1. `grooming_contract_standard_petsuites_encodes_calendar_rebooking_no_show_and_history_rules`
   - Asserts `Contract::standard_petsuites()` uses `CalendarPolicy::GroomerSpecific`, six-week rebooking, deposit-after-no-show behavior, two reminder rules, and style/photo history.
2. `grooming_service_menu_rejects_empty_or_duplicate_location_offerings`
   - `ServiceMenu` builder rejects empty menus and duplicate offerings.
3. `grooming_rebooking_cadence_accepts_two_to_eight_week_ordinary_window`
   - Ordinary recurring cadence permits 2-8 weeks and rejects 0/1/9+ unless represented by an explicit override/groomer-recommended variant.
4. `grooming_duration_estimate_requires_positive_minutes_and_explains_basis`
   - Estimate cannot be zero and carries `EstimateBasis` plus confidence/review requirement.
5. `matted_or_sensitive_coat_estimate_requires_staff_review_before_auto_scheduling`
   - Estimation policy returns staff/groomer review for matted or sensitive/medical product scenarios.
6. `groomer_specific_calendar_policy_rejects_unqualified_or_overlapping_schedule_candidate`
   - Scheduling policy explains conflict rather than producing an invalid candidate.
7. `grooming_no_show_policy_requires_deposit_or_manager_review_for_repeat_no_show`
   - No-show/cancellation history cannot be ignored or represented as a boolean.
8. `grooming_rebooking_policy_marks_pet_overdue_from_last_service_history_and_cadence`
   - Service history plus cadence produces due/overdue recommendation with typed rationale.
9. `exit_bath_cross_sell_after_boarding_is_recommendation_not_reservation_mutation`
   - Cross-sell returns a draft opportunity linked to reservation; it does not mutate reservation or payment state.
10. `grooming_reminder_plan_requires_customer_consent_before_member_facing_send`
    - Reminder drafts are created, but send execution is gated by consent/approval decision.
11. `grooming_history_entry_separates_style_notes_from_care_or_medical_handling_refs`
    - Style notes and care/medical refs are distinct typed fields; sensitive debug output is redacted if applicable.
12. `grooming_automation_policy_never_executes_booking_payment_or_customer_message_from_llm_recommendation_alone`
    - Agent output maps to draft/staff/manager review unless a typed approval token exists.

Storage/boundary tests:

1. `grooming_service_offering_records_roundtrip_all_source_service_variants`
   - Storage codecs preserve mini groom, full groom, exit bath, baths, nails, ear cleaning, products, and first-time offers.
2. `grooming_service_offering_record_rejects_cross_variant_storage_shape`
   - Existing cross-variant rejection should remain and expand for grooming-owned offerings.
3. `grooming_contract_record_rejects_zero_minutes_or_zero_cadence_weeks`
   - Current validated scalar rejection continues for nested grooming contract records.
4. `grooming_history_record_promotes_provider_note_payloads_to_semantic_history_entries`
   - Raw provider notes are converted into typed history fields or review-required parse failures.
5. `grooming_calendar_adapter_returns_draft_hold_not_confirmed_booking_without_approval`
   - Tool boundary preserves draft-vs-execution semantics.

Workflow/agent tests:

1. `grooming_rebooking_agent_returns_recommendation_with_evidence_and_review_boundary`
2. `grooming_calendar_optimizer_explains_conflicts_and_does_not_override_manager_policy`
3. `grooming_cross_sell_agent_drafts_customer_message_but_marks_member_facing_send_review_required`
4. `grooming_note_summarizer_redacts_or_refs_sensitive_care_details`
5. `grooming_reputation_signal_routes_negative_grooming_outcome_to_manager_review`

Implementation entry rule for later cards: write the failing semantic API test first, verify RED, implement the smallest grooming-owned type/policy/repository surface needed, then run the focused test and workspace gates. Do not add scheduling, reminders, or cross-sell behavior on top of raw strings, booleans, provider IDs, or generic helper functions.
