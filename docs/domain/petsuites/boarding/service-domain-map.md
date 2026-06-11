# PetSuites boarding service domain map

Purpose: model the Boarding core service domain for the NVA/PetSuites AI-operations foundation. This is a contract/spec artifact for later Rust cards; it does not authorize live reservation, payment, customer-message, or member-facing actions.

Assumptions:

- PetSuites boarding is a location-scoped service line with local policy variation. The foundation should encode a conservative NVA/PetSuites default but preserve per-location contracts.
- Dog boarding and cat boarding share the stay/reservation/payment lifecycle, but their accommodation, enrichment, play, and care-review rules differ enough to require typed variants rather than string labels.
- Public source material names classic/luxury suite types, daily housekeeping, potty walks, bedding, Pawgress Report, add-ons/upgrades, deposits/cancellation policies, and peak/holiday minimum stays. Exact prices, room counts, deadlines, and local blackout rules are provider/location data, not hard-coded constants.
- AI agents may draft, summarize, recommend, detect risk, and create internal review tasks. They must not confirm bookings, change reservations, waive/refund deposits, approve ambiguous care/medical/behavior evidence, or send sensitive customer-facing messages without explicit typed approval.

## 1. Vocabulary and bounded context

Boarding is the overnight-stay service line. Its domain boundary owns the service contract that turns a requested stay into a capacity-aware, care-aware, payment-aware, staff-operable plan.

Core vocabulary:

- Boarding stay: an overnight reservation for one or more pets at a location.
- Accommodation: the physical overnight room/suite/condo product reserved for a pet. Boarding needs distinct dog and cat accommodation variants.
- Classic suite / luxury suite: dog boarding accommodation tiers. Luxury is not just a price label; it affects availability, upgrade opportunities, premium bedding/webcam-like amenities where supported, and fulfillment expectations.
- Cat condo: cat boarding accommodation. It should not inherit dog playgroup assumptions.
- Room inventory: count of reservable accommodations at a location, segmented by accommodation type.
- Room availability: operational availability state for a room segment/date range: open, limited, waitlist-only, closed, held-for-manager-review.
- Stay nights: positive count of boarding nights. Minimum-stay policies must use this semantic scalar, not raw `u16`.
- Peak/holiday period: named demand window with minimum-stay, deposit, and cancellation variations.
- Daily housekeeping: recurring internal care task for occupied boarding accommodations.
- Potty walk: recurring dog boarding care task; not applicable to cat boarding in the same way.
- Bedding: included or premium bedding fulfillment requirement.
- Pawgress Report: end-of-stay or stay-update customer communication artifact built from staff observations/media and subject to review rules.
- Add-on / upgrade: optional paid or included service attached to a boarding stay: playtime, exit bath, grooming, premium suite, training, premium bedding, medication support, photo/video update.
- Playtime option: enrichment plan for a boarded pet. Dog group play, dog individual play, and cat individual play are different policy surfaces.
- Deposit: payment requirement tied to booking/holiday/cancellation risk.
- Cancellation policy: local rule mapping notice, season, deposit status, and manager exception review to an outcome.
- Check-in flow: operational phase where staff verify profile, care instructions, medication/feeding/behavior notes, payment/deposit status, belongings, and accommodation assignment.
- Check-out flow: operational phase where staff close open care tasks, collect payment, prepare belongings, attach Pawgress Report, and surface upsells such as exit bath, grooming rebooking, or training consult.
- Staff shift handoff: internal transition artifact that carries care obligations, exceptions, blocked tasks, and manager-review items between shifts.

Bounded-context summary:

`operations::boarding` should own Boarding's service-line contract, capacity policy, accommodation inventory, stay policy, add-on policy, care fulfillment obligations, staff handoff requirements, and AI approval boundaries. It should not own customer identity, pet master data, money primitives, general reservation lifecycle, or generic workflow/audit/event infrastructure; those remain in `entities`, `customer`, `pet`, `care`, `reservation`, `money`, `payment`, `workflow`, `policy`, `tools`, and `agents`.

## 2. Domain type inventory and semantic paths

Proposed caller-facing paths favor semantic modules with short role leaves. The examples below are intended as durable domain surfaces, not a mandate to implement every type in one code card.

### Service contract and offering paths

- `operations::boarding::Contract`
  - Location-scoped Boarding service contract.
  - Composes capacity, stay, deposit, cancellation, housekeeping, handoff, play, report, and upsell policies.
  - Existing type exists but should be expanded/refactored instead of replaced with a parallel contract.
- `operations::boarding::ServiceOffering`
  - Boarding-specific offering definition. Prefer this over the current root-level `operations::ServiceOffering::Boarding` once Boarding behavior grows.
  - Fields: `accommodation::Kind`, `IncludedCare`, `AddOnCatalog`, `UpgradeCatalog`, `PaymentPolicyRef`, `EligibilityPolicyRef`.
- `operations::boarding::OfferingCatalog`
  - Location/brand catalog of boarding offerings; resolves public/service labels into typed offerings.
- `operations::boarding::StayRequest`
  - Domain request for proposed boarding stay before capacity/eligibility/payment decisions.
- `operations::boarding::StayPlan`
  - Accepted internal plan after capacity and policy evaluation, still not necessarily confirmed in external systems.
- `operations::boarding::StayContract`
  - Immutable snapshot attached to a confirmed reservation so later policy changes do not mutate historical obligations.

### Accommodation and capacity paths

- `operations::boarding::accommodation::Kind`
  - `Dog(DogSuiteKind)`, `Cat(CatAccommodationKind)`.
- `operations::boarding::accommodation::DogSuiteKind`
  - `ClassicSuite`, `LuxurySuite`.
- `operations::boarding::accommodation::CatAccommodationKind`
  - `CatCondo`, later extensible to `CatSuite` if the product exists.
- `operations::boarding::accommodation::Assignment`
  - Typed room/suite assignment for a reservation/pet/date range.
- `operations::boarding::accommodation::RoomId`
  - Opaque location/provider room identity; should not be a raw string at domain call sites.
- `operations::boarding::capacity::Inventory`
  - Segmented inventory by accommodation kind.
- `operations::boarding::capacity::RoomCount`
  - Positive scalar for total rooms in a segment; supersedes or aliases current `boarding::RoomInventory` if per-accommodation segmentation is added.
- `operations::boarding::capacity::BookedCount`
  - Non-negative count for reserved rooms/pets.
- `operations::boarding::capacity::Availability`
  - `Open`, `Limited`, `WaitlistOnly`, `Closed`, `ManagerHold`.
- `operations::boarding::capacity::Snapshot`
  - Location/date/segment occupancy and availability evidence for agents and staff.
- `operations::boarding::capacity::Decision`
  - `Available { assignment_policy }`, `Waitlist { reason }`, `Deny { reason }`, `ManagerReview { reason }`.
- `operations::boarding::capacity::Policy`
  - Deterministic rules for capacity evaluation, holiday spikes, overbooking prohibition, and waitlist handling.
- `operations::boarding::capacity::Repository`
  - Reads/writes capacity snapshots and room assignment state through storage/provider adapters.

### Stay, seasonality, and policy paths

- `operations::boarding::stay::Nights`
  - Positive scalar; current `boarding::StayNights` can move or re-export as this.
- `operations::boarding::stay::DateRange`
  - Check-in date through check-out date with invariant: checkout follows checkin and derived nights >= 1.
- `operations::boarding::stay::MinimumStay`
  - Current type exists; should live near `stay` policy if expanded.
- `operations::boarding::stay::MinimumStayReason`
  - Current enum exists: `StandardPolicy`, `HolidayPeak`, `MultiPetOperationalBuffer`; add `LocationPolicy`, `ManagerOverrideRequired` if behavior needs it.
- `operations::boarding::season::Period`
  - Named demand period: holiday, school-break, local-event, blackout.
- `operations::boarding::season::Policy`
  - Maps dates to demand periods, minimum stays, deposits, cancellation changes, and manager holds.
- `operations::boarding::service_window::Window`
  - Current `boarding::ServiceWindow`; invariant end follows start.
- `operations::boarding::service_window::HourOfDay`
  - Current scalar; invariant 0..=23.
- `operations::boarding::cancellation::Policy`
  - Current `boarding::CancellationPolicy` can move/re-export here.
- `operations::boarding::cancellation::NoticeHours`
  - Current scalar; positive hours.
- `operations::boarding::cancellation::Penalty`
  - Current `CancellationPenalty`; add typed exception route if manager approval behavior expands.
- `operations::boarding::deposit::Rule`
  - Current `boarding::DepositRule`, composed with `money::Money` and `payment::DepositStatus` when tied to an actual reservation.
- `operations::boarding::deposit::Decision`
  - `NotRequired`, `Required { amount, due }`, `AlreadyPaid`, `ManagerReviewRequired`.

### Care fulfillment and handoff paths

- `operations::boarding::care::Requirement`
  - Boarding-specific care obligations: feeding support, medication support, potty walks, bedding, housekeeping, playtime, report artifact.
- `operations::boarding::care::Plan`
  - Per-pet fulfillment plan derived from `entities::CareProfile`, `TemperamentProfile`, add-ons, accommodation, and policy decisions.
- `operations::boarding::care::FeedingReview`
  - `Complete`, `MissingInstruction`, `ConflictingInstruction`, `ManagerOrVetReviewRequired`.
- `operations::boarding::care::MedicationReview`
  - `NoMedication`, `StaffCheckRequired`, `ManagerReviewRequired`, `VetClarificationRequired`.
- `operations::boarding::care::BehaviorReview`
  - Review outcome for anxiety, aggression, group-play, special handling, and incident history.
- `operations::boarding::task::Kind`
  - Boarding-owned staff-task kinds that can bridge into `operations::StaffTaskKind`: arrival prep, room turnover, daily housekeeping, potty walk, feeding, medication, playtime, Pawgress draft, checkout prep, belongings review.
- `operations::boarding::handoff::Packet`
  - Shift handoff summary with reservation IDs, pet IDs, incomplete tasks, blocked care items, review gates, and manager escalations.
- `operations::boarding::handoff::Requirement`
  - Existing `boarding::HandoffRequirement` can expand here.

### Add-ons, upgrades, and reports

- `operations::boarding::add_on::Kind`
  - `Playtime(PlaytimeKind)`, `ExitBath`, `Grooming`, `TrainingSession`, `PremiumBedding`, `MedicationAdministration`, `PhotoVideoUpdate`, `Other(add_on::Label)`.
- `operations::boarding::playtime::Kind`
  - `DogGroupPlay`, `DogIndividualPlay`, `CatIndividualPlay`, `PrivateEnrichment`.
- `operations::boarding::playtime::Eligibility`
  - Composes `policy::PlayEligibilityDecision`, species, temperament, spay/neuter status, and staff review.
- `operations::boarding::upgrade::Kind`
  - `LuxurySuite`, `PremiumBedding`, `WebcamOrEnhancedVisibility`, `ExtraReport`, `ExtendedCheckout` where supported.
- `operations::boarding::upsell::Opportunity`
  - Existing root `RevenueOpportunityKind` values should be mapped into Boarding-owned opportunities when tied to a stay.
- `operations::boarding::upsell::Recommendation`
  - Agent/staff recommendation with rationale, eligibility, customer-safety boundary, and approval state.
- `operations::boarding::report::PawgressReport`
  - Customer-facing report draft or approved artifact.
- `operations::boarding::report::SourceEvidence`
  - Staff notes/media/task completion evidence used to generate the report.
- `operations::boarding::report::ApprovalState`
  - `Draft`, `StaffReviewed`, `ManagerReviewRequired`, `ApprovedToSend`, `Sent`.

### Repositories and domain services

- `operations::boarding::Repository`
  - Aggregate repository for boarding contracts by `entities::LocationId` and contract version.
- `operations::boarding::catalog::Repository`
  - Offering/catalog lookup by location/accommodation/add-on.
- `operations::boarding::capacity::Repository`
  - Capacity snapshot and room assignment store.
- `operations::boarding::reservation::Repository`
  - Boarding-facing reservation query/update port that returns `entities::Reservation` plus boarding-specific projections.
- `operations::boarding::care::Repository`
  - Read care profile and task evidence; write internal care-plan/task projections only.
- `operations::boarding::policy::Evaluator`
  - Domain service that evaluates stay request against capacity, care, deposit, cancellation, seasonality, and approval gates.
- `operations::boarding::workflow::Planner`
  - Creates deterministic `workflow::RecommendedAction` and `operations::StaffTask` drafts from a Boarding decision.
- `operations::boarding::agent::ApprovalPolicy`
  - Maps proposed agent action to `policy::AutomationLevel`/`ReviewGate`.

## 3. Existing Rust/domain surface to reuse or refactor

Current reusable domain surface from `domain/src/operations.rs`:

- `operations::CoreServiceContracts`
  - Already composes `boarding::Contract`, `daycare::Contract`, `grooming::Contract`, `training::Contract`, and `retail::Contract` by `LocationId`.
  - Reuse as the location-scoped service-contract aggregate.
- `operations::CoreServiceLine::Boarding`
  - Reuse as cross-service line enum.
- `operations::ServiceOffering::Boarding { accommodation, included_care, add_ons }`
  - Reuse as the current broad catalog variant, but refactor when behavior grows. It is too flat for Boarding-specific capacity, policy, and add-on invariants.
- `operations::BoardingAccommodation::{ClassicSuite, LuxurySuite, CatCondo}`
  - Reuse as initial accommodation vocabulary. Refactor to `operations::boarding::accommodation::{Kind, DogSuiteKind, CatAccommodationKind}` when species-specific rules need type-level ownership.
- `operations::BoardingCareFeature::{DailyHousekeeping, PottyWalks, Bedding, PawgressReport, FeedingSupport, MedicationSupport}`
  - Reuse as initial included-care vocabulary. Refactor into `operations::boarding::care::Requirement` before task scheduling/care policy branches on it.
- `operations::BoardingAddOn::{Playtime, ExitBath, PremiumSuite, Grooming, TrainingSession}`
  - Reuse as initial add-on vocabulary. Refactor into `operations::boarding::add_on::Kind` with nested playtime/grooming/training/upgrade details before pricing, eligibility, or fulfillment behavior branches on it.
- `operations::boarding::Contract`
  - Already has the right service-line module path and a `bon` builder. Preserve it as the core type and expand/refactor in-place.
- `operations::boarding::RoomInventory`
  - Reuse for single segment capacity; refactor to segmented `capacity::Inventory`/`RoomCount` before modeling classic vs luxury vs cat inventory.
- `operations::boarding::StayNights`, `NoticeHours`, `HourOfDay`
  - Reuse as semantic scalars. Consider moving under `stay`, `cancellation`, and `service_window` modules if those subdomains grow.
- `operations::boarding::CapacityPlan`, `RoomAvailability`
  - Reuse as contract-level capacity posture; refactor to capacity snapshot/decision types for date-specific availability.
- `operations::boarding::ServiceWindow`
  - Reuse for check-in/check-out windows.
- `operations::boarding::MinimumStay`, `MinimumStayReason`
  - Reuse and expand to holiday/peak season contract details.
- `operations::boarding::CancellationPolicy`, `CancellationPenalty`
  - Reuse and bridge to `payment::Deposit`/manager-review gates.
- `operations::boarding::DepositRule`, `PaymentTiming`
  - Reuse for service contract defaults. Do not treat this as actual payment state; actual reservation deposit state belongs to `payment`/`entities::Reservation.deposit`.
- `operations::boarding::HousekeepingCadence`, `HandoffRequirement`, `Upsell`
  - Reuse as early policy enums; refactor under `care`, `handoff`, and `upsell` modules when behavior grows.
- `operations::StaffTask`, `StaffTaskKind`, `StaffTaskAssignment`, `StaffRole`, `StaffTaskSource`, `TaskCompletionEvidence`
  - Reuse for internal staff task representation. Add Boarding-specific mapping rather than stuffing every nuance into generic `StaffTaskKind` variants.
- `operations::ResortDailyBrief`, `DailyBriefSection`, `OccupancySnapshot`, `ArrivalDepartureSnapshot`, `PetCareWatch`, `RevenueOpportunity`, `OperationsRisk`, `OperationsAction`
  - Reuse for manager briefings and agent outputs. Boarding-specific decisions should be computed in Boarding-owned services and then summarized through these general operations types.
- `operations::AiUseCase::{PostStayPawgressReportAssistant, BoardingPreArrivalChecklistAutomation, CapacityAlerts, DemandForecasting, StaffingRecommendations, WebsiteReservationAssistant, VaccinationDocumentCollection}`
  - Reuse as strategic use-case vocabulary, not as executable policy.

Related Rust/domain surfaces:

- `entities::Location`, `LocationId`, `Brand`, `LocationPolicyRefs`, `ServiceKind::Boarding`
  - Boarding contracts are location-scoped and must respect local policy refs.
- `entities::Customer`, `CustomerId`, `ContactChannel`, `PortalAccountRef`
  - Customer identity/contact preferences are external to Boarding but required for deposits, reminders, reports, and approvals.
- `entities::Pet`, `PetId`, `Species`, `TemperamentProfile`, `CareProfile`, `MedicationInstruction`
  - Boarding consumes these for eligibility/care planning; it should not duplicate master pet profile state.
- `entities::Reservation`, `ReservationStatus`, `ReservationSource`, `AddOn`, `HardStop`
  - Boarding stay lifecycle projects through reservation; Boarding adds service-line semantics around capacity/care/payment.
- `care::*`, `temperament::*`, `policy::*`
  - Use care and temperament facts as observations; policy owns decisions/review gates.
- `money::Money`, `money::MinorUnits`, `money::Currency`, `payment::Deposit`, `payment::DepositStatus`, `payment::PaymentReference`
  - Boarding owns deposit rule semantics, payment owns actual transaction/deposit state.
- `workflow::*`, `tools::*`, `agents::*`
  - Agents recommend/draft; deterministic domain policy validates; tools execute only approved drafts.
- `storage::operations::CoreServiceContractsRecord` and storage tests
  - Existing storage codecs roundtrip `CoreServiceContracts`, including `boarding::Contract::standard_petsuites()`. Any refactor must preserve or intentionally migrate this contract serialization.

## 4. Required newtypes/enums/builders/policies/repositories/domain services

### Newtypes and scalar invariants

- `operations::boarding::stay::Nights`
  - Positive integer; zero nights is invalid.
- `operations::boarding::stay::DateRange`
  - Check-out date/time must follow check-in; derived nights must be positive; location timezone must be explicit if local dates matter.
- `operations::boarding::capacity::RoomCount`
  - Positive for configured inventory; separate `BookedCount` can be zero.
- `operations::boarding::capacity::OccupancyBasisPoints`
  - 0..=10_000 for normal saturation; if overbooking is ever represented, use a separate `OverbookedBy` rather than allowing >10_000 silently.
- `operations::boarding::accommodation::RoomId`
  - Non-empty provider/location room identifier.
- `operations::boarding::season::Name`
  - Non-empty, bounded label for holiday/peak periods.
- `operations::boarding::cancellation::NoticeHours`
  - Positive hours; already exists as `boarding::NoticeHours`.
- `operations::boarding::deposit::DueAt`
  - Semantic due timing: at booking, at check-in, at checkout, by local deadline.
- `operations::boarding::report::DraftText`, `report::ReviewNote`
  - Non-empty, bounded, redaction-aware customer-facing/staff-facing text values.

### Enums and semantic decisions

- `accommodation::Kind`, `DogSuiteKind`, `CatAccommodationKind`
  - Prevent cat boarding from inheriting dog-specific play/potty-walk defaults.
- `capacity::Availability`, `capacity::Decision`, `capacity::DenialReason`, `capacity::WaitlistReason`
  - Avoid raw availability strings and boolean `available` responses.
- `season::DemandClass`
  - `Normal`, `Peak`, `Holiday`, `Blackout`, `LocalEvent`.
- `stay::MinimumStayReason`
  - Existing enum should remain the center for minimum-stay invariants.
- `deposit::Decision`, `deposit::ExceptionReason`
  - Manager exceptions and refunds/waivers must be explicit review paths.
- `cancellation::Outcome`
  - `AllowedNoPenalty`, `AllowedForfeitDeposit`, `ManagerReviewRequired`, `DeniedAfterCheckIn`, etc.
- `care::Requirement`, `care::ReviewGate`, `care::FulfillmentStatus`
  - Care obligations and review needs must not be hidden in task titles.
- `playtime::Eligibility`, `playtime::IneligibilityReason`
  - Bridge from `policy::PlayEligibilityDecision` but keep Boarding-specific enrichment semantics.
- `handoff::Severity`
  - `Info`, `NeedsShiftLead`, `NeedsManager`, `SafetyCritical`.
- `report::ApprovalState`
  - Customer-facing Pawgress Reports cannot be sent from an undifferentiated draft string.
- `upsell::Eligibility`, `upsell::ApprovalState`
  - Prevent unsafe/inappropriate upsell recommendations.

### Builders and typestate candidates

- `operations::boarding::Contract::builder()`
  - Already exists via `bon`. Keep required fields for capacity, windows, minimum stay, cancellation, deposit/payment, housekeeping, handoff.
  - Add defaulted vectors only where absence is semantically safe: add-on catalog, upsells, report options.
- `operations::boarding::StayRequest::builder()`
  - Required: location, customer, pet IDs, requested date range, requested accommodation or flexibility, requested add-ons.
  - Build should not require capacity proof; it represents a request.
- `operations::boarding::StayPlan::builder()` or typestate
  - Consider typestate only when method legality differs by phase:
    - `Requested` -> `CapacityEvaluated` -> `CareReviewed` -> `DepositEvaluated` -> `ReadyForStaffApproval` -> `ApprovedForBooking`.
  - Do not introduce typestate just for documentation; use it when compile-time phase correctness prevents unsafe actions.
- `operations::boarding::care::Plan::builder()`
  - Required pet/profile facts, accommodation, care requirements, and review outcomes. Missing medication/feeding profile data should produce explicit review states.
- `operations::boarding::report::PawgressReport::draft()`
  - Draft creation from `SourceEvidence`; approval transition requires staff/manager actor depending on content risk.

### Policies and deterministic services

- `operations::boarding::capacity::Policy`
  - Inputs: location, date range, accommodation kind, pet count, inventory snapshot, season policy, current reservations/holds.
  - Output: `capacity::Decision` with typed reasons and evidence.
  - Invariants: never overbook without explicit `ManagerReview`; never assign cat accommodation to dogs or dog suites to cats; holiday holds must be represented as policy decisions.
- `operations::boarding::stay::Policy`
  - Enforces minimum nights, check-in/check-out windows, and local peak periods.
- `operations::boarding::care::Policy`
  - Evaluates care profile completeness, medication review, feeding exceptions, behavior/anxiety flags, playtime eligibility, and special-handling requirements.
- `operations::boarding::deposit::Policy`
  - Determines deposit requirement/timing from season, customer/reservation history if available, cancellation policy, and manager overrides.
- `operations::boarding::cancellation::Policy`
  - Evaluates cancellation/change requests with notice, deposit status, season, and checked-in state.
- `operations::boarding::upsell::Policy`
  - Suggests only eligible, care-safe add-ons. It must suppress upsells when the pet/customer context makes the suggestion insensitive or unsafe.
- `operations::boarding::agent::ApprovalPolicy`
  - Deterministically maps actions to automation levels:
    - internal read/summarize/draft tasks: safe or internal-only;
    - customer messages/reports: draft-only until review;
    - booking/payment/deposit changes: manager/staff approval required;
    - medical/behavior/incident decisions: never fully automate.

### Repositories and ports

- `operations::boarding::Repository`
  - Read/write Boarding contract snapshots by location/version.
- `operations::boarding::capacity::Repository`
  - Read inventory/availability snapshots; create internal holds only through approved workflows.
- `operations::boarding::room::Repository`
  - Room/suite assignment projections; provider room IDs stay behind semantic newtypes.
- `operations::boarding::reservation::Repository`
  - Query reservations in a date/location/service range; propose status updates as `tools::ReservationUpdateDraft` or `workflow::RecommendedAction`, not direct live changes.
- `operations::boarding::care::Repository`
  - Read care/temperament profile projections and task evidence; create/update internal staff tasks.
- `operations::boarding::payment::Repository`
  - Read deposit status/payment references; draft collection/refund/waiver requests subject to review gates.
- `operations::boarding::report::Repository`
  - Store Pawgress Report drafts, evidence references, approval state, and sent metadata.

## 5. Relationships to adjacent modules

- Customer:
  - `entities::CustomerId` anchors a stay to the paying/communicating human.
  - `entities::Customer.preferred_contact` and `PortalAccountRef` inform reminders/reports but do not authorize sending.
  - Boarding should use customer refs, not duplicate names/emails in service contracts.
- Pet:
  - `entities::PetId`, `Species`, `SpayNeuterStatus`, `TemperamentProfile`, and `CareProfile` drive accommodation, playtime eligibility, care review, and handoff tasks.
  - Boarding should preserve species-specific semantics: dog boarding and cat boarding share stay infrastructure but not all care/play rules.
- Reservation:
  - `entities::Reservation` is the cross-service lifecycle aggregate. Boarding owns service-line projections/decisions that may recommend reservation status transitions.
  - Status transitions should flow through `workflow::RecommendedAction::UpdateStatus` / `tools::ReservationUpdateDraft`, not direct mutation by agents.
- Care profile:
  - `entities::CareProfile`, `care::FeedingInstruction`, `MedicationInstruction`, allergies, medical conditions, emergency/vet contacts are consumed by `boarding::care::Policy`.
  - Missing/ambiguous care facts produce review gates and staff tasks, not guessed defaults.
- Location:
  - `entities::LocationId`, `Brand::PetSuites`, capabilities, and `LocationPolicyRefs` select contract/policy versions.
  - Local room counts, check-in/out windows, holiday periods, and deposit/cancellation rules remain location-specific.
- Staff task:
  - Boarding creates `operations::StaffTask` drafts for check-in prep, check-out prep, feeding, medication, cleaning turnover, daily updates/Pawgress drafts, document review, incident follow-up, and customer follow-up.
  - `StaffTaskKind` can stay generic, but Boarding should own the mapping from `boarding::task::Kind` to generic operations tasks.
- Money/payment:
  - `money::Money` and `payment::Deposit` own amounts/status/payment references.
  - Boarding owns policy rules like when a deposit is required, when it may be forfeited, and which exceptions need manager approval.
- Workflow/agent:
  - `workflow::WorkflowEvent`, `PolicyContext`, `RecommendedAction`, `AllowedAction`, `ReviewReason`, `RiskFlag`, and `VerificationNote` are the audit/recommendation envelope.
  - `agents::WorkflowAgent` and `agents::AgentPromptPacket` can run boarding pre-arrival, capacity alert, Pawgress draft, and upsell recommendation workflows, but domain policy remains deterministic Rust.
- Tools/storage:
  - `tools::AvailabilityRequest/Result`, `ReservationUpdateDraft`, payment/document/media/messaging ports are execution boundaries.
  - `storage::operations` persists contract snapshots; future Boarding-specific storage records should convert to/from semantic domain values rather than leaking provider JSON into core.

## 6. AI-agent opportunities and approval boundaries

### Safe-to-automate or internal-only actions

- Summarize upcoming check-ins/check-outs from read-only reservation/capacity data.
- Detect missing pet profile, vaccine, medication, feeding, behavior, or deposit requirements and create internal review task drafts.
- Draft staff shift handoff packets from open tasks, care flags, capacity risk, and reservation status.
- Draft Pawgress Report text from staff-approved notes/media/evidence, marked `Draft`.
- Recommend availability/capacity risks to managers from deterministic snapshots.
- Recommend eligible add-ons/upsells internally, with rationale and suppression reasons.
- Generate manager daily brief sections for boarding occupancy, arrivals/departures, pet-care watchlist, and revenue opportunities.

### Staff review required

- Sending Pawgress Reports or other customer-facing stay updates.
- Confirming that medication/feeding instructions are complete enough for stay fulfillment.
- Approving playtime/group-play eligibility when temperament or spay/neuter status is ambiguous.
- Marking care tasks complete when completion evidence is free-text or media-based.
- Changing check-in/check-out assumptions based on customer messages.
- Applying a room assignment when inventory is limited, segmented, or under local hold.

### Manager approval required

- Overriding capacity, room holds, waitlists, minimum-stay rules, or holiday/peak restrictions.
- Waiving/refunding/forfeiting deposits or making cancellation exceptions.
- Handling customer complaints, incidents, injuries, safety issues, or legal/regulatory signals.
- Sending sensitive customer-facing explanations about medical, behavior, incident, payment, or denial decisions.
- Approving staff schedule changes based on demand forecasts.

### Member-facing or unsafe without explicit approval

- Confirming, cancelling, modifying, or rejecting a live reservation.
- Charging, refunding, waiving, or forfeiting money.
- Sending customer messages, Pawgress Reports, legal/policy explanations, or public statements.
- Approving medical documents, medication instructions, behavior safety, or incident closure.
- Hiding negative care facts, safety risks, or unresolved staff tasks from reports.
- Making dynamic price/availability promises without deterministic capacity/payment policy and human approval.

Approval model:

- Agents produce `workflow::RecommendedAction`, `operations::StaffTask` drafts, `boarding::report::PawgressReport` drafts, and `boarding::upsell::Recommendation` values.
- `boarding::agent::ApprovalPolicy` maps each output to `policy::AutomationLevel` and optional `policy::ReviewGate`.
- Tool adapters only execute actions carrying a deterministic approval decision. Absence of a review gate is not approval.

## 7. Acceptance tests/contracts for later code cards

These are executable-contract names and behaviors future implementation cards should add before or alongside code.

### Boarding contract and catalog

- `boarding_contract_standard_petsuites_preserves_deposit_minimum_stay_housekeeping_and_handoff_rules`
  - `operations::boarding::Contract::standard_petsuites()` exposes a non-zero capacity plan, valid arrival/departure windows, positive minimum stay, deposit rule, daily housekeeping, handoff requirement, and upsells.
- `boarding_service_offering_distinguishes_dog_classic_dog_luxury_and_cat_accommodation`
  - Dog classic, dog luxury, and cat condo offerings are different semantic variants, not labels in a string.
- `boarding_add_on_catalog_preserves_playtime_exit_bath_grooming_training_and_premium_suite_meaning`
  - Add-ons/upgrades roundtrip through domain/storage without flattening to raw strings.

### Capacity and room availability

- `boarding_capacity_policy_rejects_zero_room_inventory`
  - Zero configured room inventory is rejected at construction/deserialization.
- `boarding_capacity_decision_waitlists_when_holiday_segment_is_full`
  - Full holiday inventory returns `capacity::Decision::Waitlist` or `ManagerReview`, not `Available`.
- `boarding_capacity_snapshot_segments_classic_luxury_and_cat_inventory`
  - Capacity by accommodation kind remains segmented.
- `boarding_room_assignment_cannot_assign_cat_to_dog_suite_or_dog_to_cat_condo`
  - Species/accommodation mismatch is impossible or returns a typed denial.

### Stay, deposit, cancellation, and payment

- `boarding_stay_date_range_requires_checkout_after_checkin`
  - Invalid date range cannot construct.
- `boarding_peak_period_minimum_stay_overrides_standard_minimum`
  - Holiday/peak policy returns the stricter typed minimum stay.
- `boarding_deposit_policy_requires_collection_before_confirmation_when_due_at_booking`
  - Deposit decision blocks confirmation/drafts a collection task when unpaid.
- `boarding_cancellation_policy_routes_late_holiday_cancellation_to_manager_review_or_forfeit_deposit`
  - Cancellation outcome is typed and review-gated.
- `boarding_deposit_exception_cannot_be_automated_without_manager_approval`
  - Waiver/refund/forfeit exceptions map to `ReviewGate::RefundOrDepositException` or manager approval.

### Care profile, medication/feeding/behavior, and handoffs

- `boarding_care_policy_flags_missing_feeding_instruction_for_staff_review`
  - Missing feeding instructions produce a care review/task, not a silent default.
- `boarding_medication_instruction_requires_staff_or_manager_review_before_checkin_complete`
  - Medication support creates review/check tasks tied to the stay.
- `boarding_behavior_or_anxiety_flag_blocks_unsupervised_playtime_recommendation`
  - Playtime recommendations respect temperament/behavior review.
- `boarding_handoff_packet_includes_open_medication_feeding_housekeeping_and_checkout_tasks`
  - Shift handoff carries concrete typed obligations.
- `boarding_checkout_flow_requires_belongings_payment_and_report_review_before_complete`
  - Checkout readiness evaluates payment, belongings, and report approval state.

### Staff tasks and workflows

- `boarding_prearrival_checklist_creates_internal_tasks_for_missing_profile_deposit_and_care_reviews`
  - Agent/workflow output is internal staff tasks, not live customer actions.
- `boarding_staff_task_mapping_preserves_boarding_task_kind_semantics`
  - Boarding-owned task kind maps into `operations::StaffTaskKind` without losing reason/source.
- `boarding_manager_daily_brief_surfaces_capacity_pet_care_and_revenue_risks`
  - Boarding signals summarize into `ResortDailyBrief` sections.

### Pawgress Report and customer communication

- `boarding_pawgress_report_draft_requires_source_evidence`
  - A report draft cannot be built from an empty/no-evidence prompt.
- `boarding_pawgress_report_with_medical_behavior_or_incident_content_requires_manager_review`
  - Sensitive content is review-gated.
- `boarding_pawgress_report_cannot_be_sent_from_draft_state`
  - Customer-facing send requires approval state.
- `boarding_customer_message_drafts_preserve_review_reason_and_contact_channel`
  - Draft messages carry customer contact preference and review gate.

### Upsells and revenue opportunities

- `boarding_upsell_policy_recommends_exit_bath_only_when_eligible_and_not_care_unsafe`
  - Exit bath recommendation is suppressed for incompatible care/medical/behavior states.
- `boarding_training_or_grooming_upsell_remains_internal_recommendation_until_staff_approved`
  - Upsells do not become customer-facing offers automatically.
- `boarding_revenue_opportunity_maps_to_operations_daily_brief_without_losing_boarding_context`
  - General operations summaries preserve Boarding source context.

### Agent approval boundaries

- `boarding_agent_can_draft_prearrival_tasks_but_cannot_confirm_reservation`
  - Approval policy marks confirmation as staff/manager required.
- `boarding_agent_cannot_charge_refund_or_waive_deposit`
  - Payment/deposit actions require explicit manager/staff approval.
- `boarding_agent_capacity_alert_is_recommendation_not_room_assignment_when_inventory_limited`
  - Limited/full capacity produces recommendation/review, not direct assignment.
- `boarding_agent_never_sends_member_facing_sensitive_message_without_review_gate_clearance`
  - Customer messages about medical, behavior, incident, denial, payment, cancellation, or policy decisions remain draft-only.

## Implementation sequence recommendation

1. Keep `operations::boarding::Contract` as the root service-line contract; add child modules only as behavior demands them.
2. First code slice: accommodation/capacity segmentation and contract/storage tests, because room availability and holiday demand are central to Boarding.
3. Second code slice: stay/deposit/cancellation policies, reusing `money`/`payment` and preserving manager-review gates.
4. Third code slice: care/playtime/handoff task planning, consuming `entities::CareProfile`, `TemperamentProfile`, and `policy::PlayEligibilityDecision`.
5. Fourth code slice: Pawgress Report draft/approval state and AI approval-policy tests.
6. Only introduce typestate for stay lifecycle after runtime enum contracts become insufficient to express legal phase-specific behavior.
