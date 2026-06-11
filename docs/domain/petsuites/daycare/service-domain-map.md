# PetSuites daycare service domain map

Purpose: model daycare as a first-class core service domain for the NVA/PetSuites AI-operations foundation. This map is intentionally contract-shaped, not implementation-complete: later code cards should turn the vocabulary below into semantic Rust APIs, repository contracts, deterministic policy tests, and approval-gated agent workflows.

Scope assumptions:

- "Daycare" covers daytime care products where the pet returns home the same operating day unless explicitly converted to boarding.
- Dog group play and dog day boarding are different operational products. Group play requires eligibility and playgroup assignment; day boarding is supervised individual/non-group care for dogs not suited to playgroups.
- Cat individual playtime belongs in the daycare revenue/service family but must not reuse dog group-play eligibility or ratio rules blindly.
- Source systems may expose these concepts as raw service codes, package names, notes, booleans, or calendar blocks. Domain code should promote them into semantic values before policy, pricing, workflow, or automation behavior branches on them.

## 1. Domain vocabulary and bounded-context summary

Bounded context: `operations::daycare` owns the service contract for daytime care: which product is requested, what eligibility evidence is required, how attendance recurs, how pets are assigned to care mode/groups, what staff coverage is required, what incidents or notes change eligibility, and which package/membership opportunities are safe to surface.

Neighbor contexts provide identities and external facts:

- `customer`: customer identity/contact preferences and customer-safe messaging.
- `pet`: pet identity, species, age, and stable profile facts.
- `temperament`: behavior observations, group-play observations, staff notes, and temperament ratings.
- `care`: feeding/medication/allergy/medical notes relevant to safe daily care.
- `reservation`: reservation lifecycle, day/date windows, add-ons, age thresholds, and customer-requested transitions.
- `location`: operating site, local capacity, rooms/yards, hours, staff roles, and local policy overrides.
- `money`/`payment`: prepaid packages, memberships, per-visit charges, checkout/payment timing, and refunds/credits.
- `workflow`/`agent`: draft recommendations, review gates, task creation, customer-message drafts, and tool-boundary actions.

Core vocabulary:

| Term | Meaning in this context | Notes |
| --- | --- | --- |
| All Day Play / Full-day play | A full operating-day dog group-play daycare service. | Requires group-play eligibility, capacity, staff ratio, and playgroup assignment. |
| Half-day play | A shorter dog group-play daycare service. | Same eligibility requirements as full-day play; pricing and attendance window differ. |
| Day Boarding | Daytime individual care for dogs not suited to group play. | Uses room/run capacity and individualized care tasks rather than playgroup assignment. |
| Day Play Plus Room | Hybrid service: play/enrichment plus a dedicated room/rest period. | Needs both group/individual care eligibility and room capacity semantics. |
| Cat individual playtime | Cat daycare/enrichment service. | Should model cat handling/room/enrichment requirements separately from dog group play. |
| Group-play eligibility | Deterministic decision that a pet may join group play now. | Consumes temperament, vaccines, spay/neuter policy, incident history, and staff/capacity state. |
| Temperament assessment | Staff review of observed behavior for playgroup suitability. | Evidence, not final policy by itself. |
| Playgroup assignment | Selection of group/care lane for eligible pets. | Size, energy, temperament, age, incident history, and staff coverage matter. |
| Staff-to-pet ratio | Location/service policy for minimum staffed supervision. | Must be typed; later policy should avoid branching on naked integers. |
| Daily recurring attendance | Repeated daycare reservations/check-ins under a schedule or package/membership. | Needs recurrence exceptions, missed visits, expiration, package consumption. |
| Package/membership opportunity | Commercial signal that a customer/pet pattern is likely a good fit for prepaid visits or membership. | Agent can suggest/draft, not auto-sell or change pricing. |
| Incident | Safety/behavior/health event that may trigger owner notice, staff/manager review, or suspension from group play. | Never hide concerning facts from customer/staff review packets. |

The boundary of `operations::daycare` is the operational contract and policy language. It should not own customer identity, pet identity, medical truth, calendar persistence, payment execution, or outbound messaging delivery. It should own the daytime-care concepts that connect those neighboring contexts.

## 2. Domain type inventory using semantic paths

Current implemented surface already has a `domain::operations::daycare` submodule for a high-level contract. Future code should keep daycare-specific terms under that semantic path and move cross-service concepts only when another service truly shares the concept.

Recommended public domain surface:

```rust
operations::daycare::Contract
operations::daycare::ServiceOffering
operations::daycare::ServiceVariant
operations::daycare::AttendancePolicy
operations::daycare::AttendanceWindow
operations::daycare::AttendanceRecurrence
operations::daycare::PackagePolicy
operations::daycare::PackageVisits
operations::daycare::MembershipEligibility
operations::daycare::GroupPlayEligibilityPolicy
operations::daycare::GroupPlayEligibilityDecision
operations::daycare::EligibilityRequirement
operations::daycare::EligibilityEvidence
operations::daycare::SpayNeuterPolicy
operations::daycare::VaccinationPolicyRef
operations::daycare::TemperamentEvidence
operations::daycare::IncidentHistoryPolicy
operations::daycare::CareMode
operations::daycare::GroupAssignmentRule
operations::daycare::PlaygroupAssignment
operations::daycare::PlaygroupId
operations::daycare::PlaygroupCapacity
operations::daycare::PlaygroupRoster
operations::daycare::StaffPetRatio
operations::daycare::StaffCoveragePolicy
operations::daycare::StaffCoverageDecision
operations::daycare::IncidentPolicy
operations::daycare::IncidentSeverity
operations::daycare::IncidentDisposition
operations::daycare::DailyCareNoteRequirement
operations::daycare::DailyRecurringAttendance
operations::daycare::ReservationRequest
operations::daycare::ReservationDecision
operations::daycare::FrontDeskThroughputPolicy
operations::daycare::FrontDeskReadiness
operations::daycare::RevenueOpportunity
operations::daycare::Repository
operations::daycare::PolicyRepository
operations::daycare::RosterRepository
operations::daycare::AttendanceRepository
operations::daycare::IncidentRepository
operations::daycare::AvailabilityService
operations::daycare::EligibilityService
operations::daycare::AssignmentService
operations::daycare::PackageOpportunityService
operations::daycare::error::Error
operations::daycare::Result<T>
```

Recommended enum shapes:

```rust
operations::daycare::ServiceVariant::{
    AllDayPlay,
    HalfDayPlay,
    DayBoarding,
    DayPlayPlusRoom,
    CatIndividualPlaytime,
}

operations::daycare::CareMode::{
    DogGroupPlay,
    DogIndividualDayBoarding,
    DogHybridPlayAndRoom,
    CatIndividualEnrichment,
}

operations::daycare::GroupPlayEligibilityDecision::{
    Eligible { evidence: EligibilityEvidence },
    NeedsStaffReview { reasons: Vec<EligibilityReviewReason> },
    Ineligible { reasons: Vec<EligibilityDenialReason> },
    TemporarilySuspended { incident_id: entities::IncidentId },
}
```

Do not model this as `is_group_play: bool`, `eligible: bool`, `service_type: String`, `ratio: u16`, or `notes: Vec<String>`. A daycare reservation can be safe for individual care while unsafe for group play; that distinction must be visible in types and paths.

## 3. Existing Rust/domain surface to reuse or refactor

Current reusable surface in `domain/src/operations.rs`:

- `operations::ServiceOffering::Daycare { format, eligibility_rules }` with `operations::DaycareFormat::{AllDayPlay, HalfDayPlay, DayBoarding, DayPlayPlusRoom, CatIndividualPlaytime}` and `operations::DaycareEligibilityRule::{TemperamentReviewRequired, SpayNeuterRequiredForGroupPlay, VaccineProofRequired, StaffToPetRatioRequired}`.
- `operations::daycare::Contract` with `AttendancePolicy`, `PackagePolicy`, `PackageVisits`, `StaffPetRatio`, `StaffCount`, `PetCount`, `GroupAssignmentRule`, `IncidentPolicy`, and `EligibilityRequirement`.
- `operations::daycare::Contract::standard_petsuites()` and `requires_staff_review_before_group_play()`.
- `operations::CoreServiceContracts` composes `boarding`, `daycare`, `grooming`, `training`, and `retail` contracts per `LocationId`.
- `OperationsRisk::{CapacityConstraint, LaborMismatch, CustomerExperienceRisk, PetSafetyOrCareRisk, RevenueLeakage}` and `OperationsAction::{CreateInternalTask, DraftCustomerMessage, EscalateToManager, SuggestScheduleReview, SuggestRevenueFollowUp}` can carry daycare operational signals.
- `RevenueOpportunityKind::DaycarePackageCandidate`, `FollowUpReason::{MissingVaccineProof, ReservationChangeRequested}`, and `StaffTaskKind::{PlaygroupAssessment, IncidentFollowUp, DailyUpdateDraft, DocumentReview, CustomerFollowUp}` are directly relevant.

Current related modules to reuse:

- `temperament.rs`: `GroupPlayObservation`, `PeopleOrientation`, `TemperamentRating`, `BehaviorObservation`, `StaffNote`, and redacted debug values.
- `care.rs`: feeding, medical/allergy/medication notes and review requirements.
- `reservation/mod.rs`: `MinimumAgeWeeks`, `AgeThreshold`, `AgePolicyReason::{DayPlayMinimum, DaycareMinimum}`, `AddOnLabel`, and transition reasons.
- `policy.rs`: `ReviewGate`, `PlayEligibility`, conservative play eligibility policy, automation levels/rules, policy IDs, vaccine names, and denial reasons.
- `agents.rs`: baseline `booking-triage`, `daily-care-update`, `incident-escalation`, `manager-daily-brief`, and `lead-conversion` specs.
- `tools.rs`: availability lookup, reservation draft/update, portal, payment, messaging, document intake, media snapshot, and Hermes task boundaries.

Refactor guidance:

1. Keep `operations::daycare::Contract` as the location-level service contract.
2. Move the top-level `operations::DaycareFormat` and `operations::DaycareEligibilityRule` under `operations::daycare` as `ServiceVariant`/`EligibilityRequirement` or re-export them from `operations::daycare` if backwards compatibility is needed. The path should say daycare at the call site.
3. Split `GroupAssignmentRule` into the rule/policy (`TemperamentAndSizeMatched`, `IndividualPlayOnly`, `SeniorOrLowEnergyGroup`) and the concrete assignment (`PlaygroupAssignment { group_id, care_mode, rationale, staff_coverage }`).
4. Promote group-play eligibility from requirement lists into a policy decision that consumes typed evidence and returns explicit eligible/review/ineligible/suspended outcomes.
5. Keep `StaffPetRatio` but add policy-level threshold/decision types before it drives scheduling or booking limits.
6. Add repository traits only when persistence/adapters need them; place behavior on daycare-owned repositories/services, not in generic helpers.

## 4. Required newtypes, enums, builders, policies, repositories, and domain services

Newtypes/scalars:

- `operations::daycare::PackageVisits(u16)`: already exists; non-zero invariant.
- `operations::daycare::StaffCount(u16)` / `PetCount(u16)`: already exists; non-zero invariant.
- `operations::daycare::PlaygroupId`: non-empty provider-neutral identifier; source-system IDs convert at the boundary.
- `operations::daycare::RosterLimit`: non-zero upper bound for a playgroup or room/care lane.
- `operations::daycare::AttendanceDays`: non-empty set or recurrence rule, not a free-text schedule.
- `operations::daycare::MembershipTerm`: start/end/visit-window semantics; no negative or zero-length term.
- `operations::daycare::EligibilityNote` / `AssignmentRationale`: redacted debug if it can contain staff notes or sensitive behavior detail.
- `operations::daycare::IncidentId` or reuse an entity-level incident ID once incident aggregates exist.

Enums:

- `ServiceVariant`: the source service surface listed in the task.
- `CareMode`: operational care lane derived from the service variant and eligibility.
- `EligibilityRequirement`: temperament assessment, vaccines current, spay/neuter for group play, staff ratio available, age threshold met, incident review cleared, manager override if allowed.
- `EligibilityReviewReason`: missing temperament assessment, stale assessment, missing vaccine proof, uncertain spay/neuter status, incident pending review, capacity/ratio uncertain, medical/care note requires staff review.
- `EligibilityDenialReason`: not spayed/neutered where required, vaccines not current, age below minimum, bite/aggression hard stop, suspended pending manager review, service unavailable for species/care mode.
- `SpayNeuterPolicy`: required for group play, waived by age/medical/manager exception, not applicable to individual/cat care.
- `StaffCoverageDecision`: sufficient, insufficient, unknown, manager override required.
- `IncidentSeverity`: note-only, owner-notice, manager-review, suspend-group-play, emergency/vet/escalation.
- `FrontDeskReadiness`: ready to check in, missing requirements, staff review required, payment/package issue, capacity waitlist.

Builders:

- `operations::daycare::Contract::builder()` should require attendance, package/membership policy, default staff ratio, assignment rule, incident policy, and eligibility policy references.
- `operations::daycare::ReservationRequest::builder()` should require customer, pet, location, service variant, requested date/window, and source; optional package/membership reference and notes default safely.
- `operations::daycare::EligibilityEvidence::builder()` should require pet, service variant/care mode, temperament evidence, vaccine status, spay/neuter status when group play is possible, current incident restrictions, and staff/capacity snapshot. Missing fields should produce `NeedsStaffReview`, not accidental eligibility.
- `operations::daycare::PlaygroupAssignment::builder()` should require eligible decision, care mode, group/room, staff coverage, and assignment rationale.

Policies/domain services:

- `GroupPlayEligibilityPolicy::evaluate(evidence) -> GroupPlayEligibilityDecision`.
  - Invariants: group play is never eligible without current temperament evidence, vaccine satisfaction, applicable spay/neuter satisfaction, no unresolved suspending incident, and sufficient staff/capacity evidence.
  - Unknown evidence maps to review, not eligible.
- `StaffCoveragePolicy::evaluate(roster, scheduled_staff, contract_ratio) -> StaffCoverageDecision`.
  - Invariants: ratio numerator/denominator are non-zero; insufficient staff cannot produce an eligible group-play assignment.
- `AssignmentService::assign(request, eligibility, capacity) -> ReservationDecision`.
  - Invariants: day boarding/cat individual care must not be forced into dog group-play paths; Day Play Plus Room must check both room and play/enrichment capacity.
- `DailyAttendanceService::materialize(recurrence, exceptions) -> Vec<ReservationRequest>`.
  - Invariants: recurrence has explicit start/end or membership/package term; holidays/blackouts/capacity closures create review or waitlist decisions.
- `PackageOpportunityService::classify(attendance_history, payment_history) -> RevenueOpportunity`.
  - Invariants: suggestion only; no auto-enrollment, payment capture, discount, or customer-facing promise.
- `FrontDeskThroughputPolicy::check(readiness_context) -> FrontDeskReadiness`.
  - Invariants: fast check-in is allowed only when requirements, payment/package state, care notes, and assignment are already resolved.

Repositories:

- `operations::daycare::PolicyRepository`: loads location-specific contract, eligibility policy, spay/neuter/vaccine rule references, ratio thresholds, and package rules.
- `operations::daycare::AttendanceRepository`: reads/writes daycare reservation/attendance facts and recurring attendance materialization results.
- `operations::daycare::RosterRepository`: reads current playgroups/rooms, capacities, staff assignments, and planned check-ins.
- `operations::daycare::IncidentRepository`: reads unresolved daycare incidents/restrictions and appends incident follow-up tasks/dispositions.
- `operations::daycare::EligibilityRepository`: stores current eligibility decisions/evidence snapshots and invalidates them when incidents, health, vaccination, spay/neuter, or temperament facts change.

All repositories should return semantic IDs, decisions, snapshots, and module-local errors. Storage DTOs may use provider codes/raw fields but must convert at adapter boundaries.

## 5. Relationships to neighboring modules

Customer:

- Use `entities::CustomerId` / `customer::{Name, Email, Phone}` for identity/contact.
- Customer-safe outputs are drafts until `policy::ReviewGate::CustomerMessageApproval` is satisfied.
- Package/membership opportunities link to customers but should remain recommendation values until staff approves offer language and pricing.

Pet:

- Use `entities::PetId`, `pet::Name`, species, age thresholds, and profile facts.
- Daycare eligibility is per pet and per care mode/service variant; a pet can be eligible for day boarding while ineligible for group play.

Reservation:

- `operations::daycare::ReservationRequest` should convert to/from `entities::Reservation` or future `reservation::daycare` contracts.
- Daily recurring attendance should materialize reservation candidates with explicit exceptions rather than mutating calendars invisibly.
- Status transitions should reuse `workflow::status_update::Target::Reservation` and typed transition reasons.

Care profile:

- Use `care::*` for feeding, medication, allergies, medical notes, emergency contacts, and review requirements.
- Medical/care uncertainty routes to staff review; agents may summarize but must not diagnose or override medical handling instructions.

Temperament:

- Use `temperament::*` as evidence for group-play policy. Temperament observations are not final decisions.
- `BehaviorObservation::{BiteHistory, RequiresManagerReview, HumanSelective}` should feed review/denial decisions conservatively.

Location:

- `LocationId` scopes the daycare contract, hours, capacity, local spay/neuter/vaccine requirements, staffing plans, and package availability.
- Location-specific overrides belong in daycare policy/config contracts, not scattered `if location == ...` branches.

Staff task:

- Use `operations::StaffTaskKind::PlaygroupAssessment` for temperament/assignment reviews.
- Use `StaffTaskKind::IncidentFollowUp` and `StaffTaskKind::DocumentReview` for incident/vaccine/spay-neuter evidence reviews.
- Front-desk readiness can create `CheckInPrep` or `CustomerFollowUp` tasks but should not silently clear hard stops.

Money/payment:

- Use `money::Money`, `payment::*`, and package/membership semantic values for checkout and package consumption.
- Auto-capture, refunds, waivers, discounts, or membership enrollment require explicit payment/customer approval gates.

Workflow/agent:

- `booking-triage` can classify daycare requests and missing requirements.
- `daily-care-update` can draft customer updates from staff-approved notes/photos.
- `incident-escalation` can summarize incident facts and create manager review packets.
- `manager-daily-brief` can surface capacity/ratio risks, unresolved eligibility reviews, incidents, and package opportunities.
- `lead-conversion` can identify daycare trial candidates and draft follow-up questions.

## 6. AI-agent opportunities and approval boundaries

Safe to automate as read/draft/recommend actions:

- Extract daycare intent from inquiries: all-day, half-day, day boarding, Day Play Plus Room, or cat playtime.
- Identify missing intake facts: pet profile, vaccines, spay/neuter status, temperament assessment, care notes, package/membership preference, requested recurrence.
- Produce front-desk readiness summaries for staff: `Ready`, `Missing vaccine proof`, `Needs playgroup assessment`, `Payment/package issue`, `Capacity/waitlist`.
- Suggest internal staff tasks: playgroup assessment, document review, incident follow-up, care-note review, package opportunity review.
- Summarize attendance patterns and suggest package/membership candidates.
- Draft customer follow-up messages with safe wording and explicit unresolved requirements.
- Draft manager daily brief sections for daycare capacity, staff ratio, incidents, and revenue opportunities.

Requires staff review:

- Initial temperament assessment and any playgroup assignment that depends on observed behavior.
- Clearing `NeedsStaffReview` eligibility reasons.
- Confirming spay/neuter/vaccine evidence when source data is ambiguous or customer-provided text is insufficient.
- Front-desk exception handling for missing documentation, late arrivals, capacity waitlist, or uncertain care notes.
- Any care instruction that affects feeding, medication, allergies, health/behavior notes, or safe handling.

Requires manager/human approval:

- Overriding group-play ineligibility, ratio/capacity limits, spay/neuter policy, vaccine policy, or incident restrictions.
- Suspending or reinstating group-play eligibility after an incident.
- Sending incident, health concern, or potentially sensitive behavior messages to customers.
- Applying refunds, waivers, discounts, membership enrollment, or package corrections.
- Changing staff schedules or staffing levels based on ratio/capacity recommendations.

Unsafe/member-facing actions agents must not perform autonomously:

- Confirm a daycare reservation or promise availability without deterministic availability and required approval.
- Auto-enroll a customer in a package or membership, capture payment, waive fees, or apply discounts.
- Hide concerning behavior/health/incident facts from staff or customer-review packets.
- Diagnose medical or behavioral conditions.
- Override local policy, manager decisions, or legal/safety gates.
- Send owner-facing incident/health/safety messages without the configured review gate.

## 7. Acceptance tests/contracts for later code cards

Contract/API tests should be written before implementation. Suggested tests:

1. `all_day_play_requires_group_play_eligibility_before_assignment`
   - Given a full-day play request with missing temperament evidence, `GroupPlayEligibilityPolicy` returns `NeedsStaffReview`, and assignment cannot produce `DogGroupPlay`.

2. `day_boarding_allows_individual_care_when_group_play_is_denied`
   - Given a dog with group-play denial but no individual-care hard stop, the decision can route to `CareMode::DogIndividualDayBoarding` with room/care capacity requirements.

3. `spay_neuter_policy_only_blocks_group_play_when_configured`
   - A missing/negative spay-neuter fact blocks `DogGroupPlay` under `SpayNeuterPolicy::RequiredForGroupPlay` but does not automatically block cat individual playtime or day boarding unless another policy says so.

4. `unknown_vaccine_or_temperament_state_routes_to_review_not_eligibility`
   - Unknown source facts produce `NeedsStaffReview` with typed reasons, never `Eligible`.

5. `staff_pet_ratio_uses_non_zero_semantic_counts`
   - `StaffCount`, `PetCount`, and roster/capacity values reject zero where nonsensical and preserve the ratio path at call sites.

6. `insufficient_staff_coverage_blocks_group_assignment`
   - If scheduled staff cannot satisfy the contract ratio for the planned roster, `StaffCoverageDecision::Insufficient` prevents assignment and creates/suggests a manager/staff review task.

7. `incident_suspension_invalidates_current_group_play_eligibility`
   - A suspending incident changes the pet's group-play decision to `TemporarilySuspended` until manager review clears it.

8. `daily_recurring_attendance_materializes_explicit_reservation_candidates`
   - A recurrence produces dated reservation candidates with exceptions/waitlist decisions; it does not silently book or check in pets.

9. `day_play_plus_room_checks_both_room_and_play_capacity`
   - Hybrid care cannot be confirmed unless both the room/rest lane and the enrichment/play lane have capacity and coverage.

10. `cat_individual_playtime_does_not_reuse_dog_group_play_rules`
    - Cat service variants use cat individual enrichment/care requirements and do not require dog playgroup assignment or dog spay/neuter group-play policy unless a location-specific policy explicitly says so.

11. `front_desk_ready_state_requires_resolved_requirements`
    - A check-in is `Ready` only when eligibility, care notes, payment/package state, capacity, and assignment are resolved. Missing documentation or unresolved notes produce typed non-ready reasons.

12. `package_candidate_is_recommendation_not_payment_action`
    - Attendance history can produce `RevenueOpportunity::PackageCandidate`, but no payment authorization, enrollment, discount, or customer message send occurs without approval.

13. `daycare_contract_roundtrips_through_storage_without_raw_string_branching`
    - Storage codecs preserve semantic variants/policies and reject invalid zero counts or unknown required enum values instead of defaulting silently.

14. `customer_message_drafts_for_daycare_require_review_gate`
    - Agent-generated follow-ups and incident/health/safety messages carry `ReviewGate::CustomerMessageApproval`; manager/safety escalations carry manager approval where required.

15. `service_variant_paths_preserve_daycare_meaning`
    - Public API examples use `operations::daycare::ServiceVariant`, `operations::daycare::EligibilityRequirement`, and `operations::daycare::Contract` rather than top-level raw daycare strings/booleans.

Implementation order recommendation:

1. Add semantic API tests for `operations::daycare::ServiceVariant`, eligibility decisions, and staff coverage.
2. Move/re-export top-level daycare format/rule enums under `operations::daycare` without breaking existing storage tests.
3. Implement policy/domain-service structs with module-local errors.
4. Add storage DTO conversion tests for all new enum/newtype variants.
5. Wire workflow/agent recommendation contracts after deterministic daycare policies are green.

Doc-only status: this artifact maps the intended domain shape. It does not change code, storage schemas, live systems, member-facing data, or operational policy.