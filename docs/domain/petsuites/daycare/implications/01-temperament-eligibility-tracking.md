# Daycare implication 01: temperament / eligibility tracking

Purpose: define the operational and domain contract for tracking a pet's daycare temperament evidence and current eligibility for PetSuites-style daycare care modes. This is a modeling artifact for later Rust code and agent workflow cards; it does not change live policy, calendars, customer communication, or member-facing state.

Scope assumption: a pet's temperament record is evidence, not the final decision. `operations::daycare` owns the eligibility decision for daycare care modes by combining temperament evidence with vaccination, spay/neuter, incident, age, care-note, capacity, and staff-coverage facts. Unknown or stale facts route to staff review rather than accidental eligibility.

## 1. Operational story

### Trigger

Temperament / eligibility tracking starts when any of these events occurs:

- A customer requests all-day play, half-day play, Day Play Plus Room, day boarding, cat individual playtime, or recurring daycare attendance.
- A pet has no current group-play assessment or has a stale temperament assessment for the requested care mode.
- A staff member records a new temperament observation, playgroup assessment, behavior note, incident, medical/care handling note, vaccination document, or spay/neuter update.
- A scheduled daycare check-in needs front-desk readiness before arrival.
- A manager reviews a suspending incident or clears/restricts group-play eligibility.

### Actors

- Customer or member: requests daycare, supplies pet facts/documents, receives approved follow-up language.
- Front desk / booking staff: gathers missing facts, reviews readiness, creates assessment/document tasks, proposes non-sensitive customer follow-up drafts.
- Daycare attendant / playgroup assessor: observes behavior, records temperament evidence, proposes playgroup/care-mode fit.
- Manager: approves overrides, suspensions, reinstatements, incident dispositions, and sensitive customer messaging.
- Domain policy engine: deterministically evaluates typed evidence into `GroupPlayEligibilityDecision` and `FrontDeskReadiness`.
- Agent workflows: read, summarize, draft, and recommend; they do not approve behavior decisions or send sensitive messages autonomously.

### Inputs

- `entities::PetId`, `entities::CustomerId`, `entities::LocationId`, requested `operations::daycare::ServiceVariant`, requested date/window, and recurrence/package context when applicable.
- `temperament::GroupPlayObservation`, `temperament::TemperamentRating`, `temperament::PeopleOrientation`, `temperament::BehaviorObservation`, and redacted `temperament::StaffNote` values.
- Assessment metadata: assessor/staff id, observed date/time, observed care mode, assessment freshness policy, and location/site context.
- Health/safety facts: vaccine policy satisfaction, spay/neuter status when group play is possible, age threshold, care-note review requirements, unresolved incidents, and manager restrictions/clearances.
- Operational snapshot: playgroup/room capacity, staff coverage, existing roster, ratio policy, service availability, and local policy refs.

### Decisions

The workflow makes separate, typed decisions rather than one broad `eligible: bool`:

1. Does the requested service require dog group-play eligibility, individual day-boarding readiness, hybrid play-and-room readiness, or cat individual enrichment readiness?
2. Is the temperament evidence current, observed by staff, and relevant to the requested care mode/location?
3. Do hard-stop behavior or incident facts deny group play, or do they require staff/manager review?
4. Are vaccines, spay/neuter policy, age thresholds, care notes, and local policy requirements satisfied or unresolved?
5. Is current staff/capacity evidence sufficient for the requested care mode?
6. Is the final state `Eligible`, `NeedsStaffReview`, `Ineligible`, or `TemporarilySuspended` for group play, and separately what non-group daycare options remain possible?
7. What internal task, review gate, audit entry, and customer-safe draft (if any) should be produced?

### Outputs

- An immutable `operations::daycare::EligibilitySnapshot` that records the typed evidence, policy version/ref, actor/source, evaluation time, and decision.
- A current `operations::daycare::GroupPlayEligibilityDecision` for dog group-play paths.
- A `operations::daycare::CareModeReadinessDecision` for individual day boarding, hybrid play-and-room, or cat individual enrichment where applicable.
- Staff tasks such as `operations::StaffTaskKind::PlaygroupAssessment`, `DocumentReview`, `IncidentFollowUp`, `CustomerFollowUp`, or a future `CheckInPrep` task.
- Workflow events such as `daycare::EligibilityReviewRequested`, `daycare::EligibilityUpdated`, `daycare::GroupPlaySuspended`, `daycare::GroupPlayReinstatementRequested`, and `daycare::FrontDeskReadinessChanged`.
- Draft customer follow-up text only when it is safe and still review-gated; no autonomous incident/safety/health messaging.

### Success state

A pet has a current, auditable daycare eligibility state for the requested service and date:

- Group-play services have an explicit group-play decision with current temperament evidence or a typed review/denial/suspension reason.
- Individual care/day boarding/cat enrichment options are evaluated on their own care-mode requirements instead of inheriting dog group-play rules.
- Front desk can see the next required action and why: ready, staff assessment needed, document review needed, manager review needed, unavailable/capacity waitlist, or customer follow-up needed.
- Any customer-facing communication remains a draft until the configured review gate is satisfied.

### Failure and exception states

- Missing or stale temperament evidence: `NeedsStaffReview { reason: MissingCurrentTemperamentAssessment | StaleTemperamentAssessment }` and a playgroup assessment task.
- Ambiguous source data: unknown vaccine, spay/neuter, age, care-note, policy, or source-system service code routes to review, not eligibility.
- Suspending behavior or incident: `TemporarilySuspended { incident_id }` or `Ineligible { reason: SafetyHardStop }`, with manager review required for reinstatement.
- Capacity/staff uncertainty: no group assignment may be produced; front desk sees capacity/ratio review or waitlist.
- Care-note or medical uncertainty: internal review task; agents may summarize but not diagnose or override handling instructions.
- Customer disputes or sensitive behavior details: manager/staff review packet; no autonomous customer send.
- Adapter/storage mismatch: reject or quarantine unknown enum/string values rather than silently defaulting to eligible.

## 2. Domain types to add or refine

### New/refined public paths

```rust
operations::daycare::ServiceVariant
operations::daycare::CareMode
operations::daycare::EligibilitySnapshot
operations::daycare::EligibilitySnapshotId
operations::daycare::EligibilityEvidence
operations::daycare::EligibilityEvidenceSource
operations::daycare::TemperamentEvidence
operations::daycare::TemperamentAssessmentId
operations::daycare::TemperamentAssessmentFreshness
operations::daycare::TemperamentAssessmentAge
operations::daycare::AssessmentObservedAt
operations::daycare::Assessor
operations::daycare::GroupPlayEligibilityPolicy
operations::daycare::GroupPlayEligibilityDecision
operations::daycare::EligibilityReviewReason
operations::daycare::EligibilityDenialReason
operations::daycare::EligibilitySuspensionReason
operations::daycare::CareModeReadinessDecision
operations::daycare::CareModeReadinessReason
operations::daycare::SpayNeuterPolicy
operations::daycare::VaccinationPolicyRef
operations::daycare::IncidentRestriction
operations::daycare::IncidentRestrictionStatus
operations::daycare::StaffCoveragePolicy
operations::daycare::StaffCoverageDecision
operations::daycare::PlaygroupAssignment
operations::daycare::AssignmentRationale
operations::daycare::EligibilityRepository
operations::daycare::TemperamentAssessmentRepository
operations::daycare::PolicyRepository
operations::daycare::RosterRepository
operations::daycare::AuditRepository
operations::daycare::error::Error
operations::daycare::Result<T>
```

### Explicit invariants

- `EligibilitySnapshotId`: non-empty provider-neutral id; source system ids convert at adapter boundaries.
- `TemperamentAssessmentId`: non-empty id for staff observation/assessment evidence; not interchangeable with `PetId` or `IncidentId`.
- `AssessmentObservedAt`: timestamp of the staff observation; cannot be in the future at creation time.
- `TemperamentAssessmentAge`: derived duration from `AssessmentObservedAt` and request date; cannot be negative.
- `TemperamentAssessmentFreshness`: enum-centered state, for example `Current`, `Stale`, `Unknown`, instead of date arithmetic spread across callers.
- `TemperamentEvidence`: contains observation/rating/behavior facts plus source and freshness; debug output must not leak raw staff notes.
- `EligibilityEvidence`: complete evidence bundle for policy evaluation; missing evidence is represented as typed unknown/review reasons, not omitted fields that default to success.
- `GroupPlayEligibilityDecision`: closed enum with `Eligible`, `NeedsStaffReview`, `Ineligible`, and `TemporarilySuspended`. It must be specific to dog group play.
- `CareModeReadinessDecision`: separate from group-play eligibility so a dog can be ineligible for group play but ready for individual day boarding.
- `EligibilityReviewReason` and `EligibilityDenialReason`: semantic enum values, not strings; reasons should carry typed context where useful.
- `SpayNeuterPolicy`: applies to group play when configured; must not automatically block cat individual enrichment or individual day boarding unless local policy says so.
- `IncidentRestriction`: unresolved suspending restrictions invalidate current group-play eligibility until manager disposition.
- `StaffCoverageDecision`: insufficient or unknown coverage cannot produce an eligible group-play assignment.
- `EligibilitySnapshot`: immutable once written; correction means append a new snapshot/audit entry, not mutating history in place.
- `AssignmentRationale` and any staff behavior-note wrapper: redacted `Debug` if it can contain sensitive behavioral detail.

## 3. Relationship map between types

### Entities

- `entities::PetId`: subject of eligibility; one pet can have different decisions per care mode and date.
- `entities::CustomerId`: owner/customer for review-gated follow-up drafts and package context.
- `entities::LocationId`: scopes policy refs, staff/capacity, local vaccine/spay-neuter requirements, and assessment freshness windows.
- `entities::StaffId`: assessor, reviewer, manager approver, or task assignee.
- `entities::ReservationId`: optional link from an eligibility snapshot to a requested/scheduled daycare visit.
- `operations::daycare::EligibilitySnapshot`: daycare-owned audit entity that ties evidence, policy, decision, source, and review gate together.

### Value objects and enums

- `operations::daycare::ServiceVariant` selects the product surface: all-day play, half-day play, day boarding, Day Play Plus Room, or cat individual playtime.
- `operations::daycare::CareMode` selects the operational lane: dog group play, dog individual day boarding, dog hybrid play-and-room, or cat individual enrichment.
- `operations::daycare::TemperamentEvidence` wraps `temperament::*` values with assessment id, source, observed-at, staff source, and freshness.
- `operations::daycare::EligibilityEvidence` aggregates temperament, vaccines, spay/neuter, age, incidents, care-note review, location policy, and staff/capacity snapshot.
- `operations::daycare::GroupPlayEligibilityDecision` and `CareModeReadinessDecision` express policy outputs.
- `operations::daycare::PlaygroupAssignment` consumes an eligible group-play decision plus roster/staff state; it is not a synonym for eligibility.

### Policies

- `operations::daycare::GroupPlayEligibilityPolicy` owns dog group-play eligibility rules.
- `operations::daycare::CareModeReadinessPolicy` owns non-group care-mode readiness rules.
- `operations::daycare::StaffCoveragePolicy` owns ratio/capacity sufficiency.
- `operations::daycare::IncidentPolicy` owns incident disposition implications: note-only, manager-review/customer-notice, or suspend group play.
- `policy::ReviewGate` remains the cross-domain approval vocabulary for manager, behavior, medical/document, and customer-message review.

### Repositories and stores

- `operations::daycare::EligibilityRepository`: stores and loads current and historical eligibility snapshots by pet/location/care mode/date.
- `operations::daycare::TemperamentAssessmentRepository`: reads staff observations and assessment records; may write new daycare assessment records after staff entry.
- `operations::daycare::PolicyRepository`: loads location-specific daycare contract, assessment freshness window, spay/neuter/vaccine refs, incident policy, and ratio policy.
- `operations::daycare::RosterRepository`: reads playgroups/rooms/staffing and current capacity snapshots.
- `operations::daycare::AuditRepository`: appends review, override, reinstatement, and customer-message approval events.
- Storage adapters may speak provider codes/raw rows; repositories expose semantic values and module-local errors.

### Workflow events

- `daycare::TemperamentAssessmentRecorded { pet, location, assessment }`
- `daycare::EligibilityReviewRequested { pet, requested_service, reasons }`
- `daycare::EligibilitySnapshotRecorded { snapshot }`
- `daycare::GroupPlaySuspended { pet, incident }`
- `daycare::GroupPlayReinstatementRequested { pet, incident }`
- `daycare::FrontDeskReadinessChanged { reservation, readiness }`
- `daycare::CustomerFollowUpDrafted { customer, review_gate }`

### Staff tasks

- `operations::StaffTaskKind::PlaygroupAssessment`: initial or stale temperament review.
- `operations::StaffTaskKind::DocumentReview`: vaccine/spay-neuter/document ambiguity.
- `operations::StaffTaskKind::IncidentFollowUp`: suspending incident or reinstatement packet.
- `operations::StaffTaskKind::DailyUpdateDraft`: safe daycare update draft after staff-approved notes.
- `operations::StaffTaskKind::CustomerFollowUp`: missing facts or approved non-sensitive follow-up.
- Future `CheckInPrep`: unresolved readiness before arrival if added later.

### Agent specs and tools

- `agents::booking-triage`: classifies daycare intent, gathers missing facts, and recommends review tasks.
- `agents::daily-care-update`: drafts updates only from staff-approved notes/photos.
- `agents::incident-escalation`: summarizes incident facts and manager review packets.
- `agents::manager-daily-brief`: surfaces unresolved eligibility reviews, suspensions, capacity/ratio risks, and reinstatement queues.
- `tools::availability_lookup`: read-only capacity/schedule facts.
- `tools::reservation_draft`: draft only; cannot confirm group play without eligibility/readiness.
- `tools::document_intake`: collect vaccine/spay-neuter evidence for staff review.
- `tools::hermes_task`: create internal staff/manager tasks.
- Messaging/payment/reservation update tools remain approval-gated for customer-facing or financial effects.

## 4. Interaction contract

Behavior should sit on the semantic owner: policies evaluate, repositories load/store, domain services orchestrate workflows, and entities/value objects protect their own invariants. Avoid free-floating helper functions such as `is_eligible(pet)` or raw `bool` flags.

```rust
impl operations::daycare::ServiceVariant {
    pub fn required_care_mode(&self, species: entities::Species) -> daycare::Result<CareMode>;
    pub fn requires_group_play_eligibility(&self) -> bool;
}

impl operations::daycare::TemperamentEvidence {
    pub fn from_assessment(
        assessment: temperament::AssessmentRecord,
        requested_at: daycare::AssessmentContext,
        freshness_policy: daycare::AssessmentFreshnessPolicy,
    ) -> daycare::Result<Self>;

    pub fn freshness(&self) -> daycare::TemperamentAssessmentFreshness;
    pub fn has_behavior_review_flag(&self) -> bool;
}

pub trait operations::daycare::PolicyRepository {
    fn daycare_contract(&self, location: entities::LocationId) -> daycare::Result<daycare::Contract>;
    fn group_play_policy(&self, location: entities::LocationId) -> daycare::Result<daycare::GroupPlayEligibilityPolicy>;
    fn assessment_freshness_policy(&self, location: entities::LocationId) -> daycare::Result<daycare::AssessmentFreshnessPolicy>;
}

pub trait operations::daycare::TemperamentAssessmentRepository {
    fn current_for_pet(
        &self,
        pet: entities::PetId,
        location: entities::LocationId,
        as_of: daycare::AssessmentObservedAt,
    ) -> daycare::Result<Option<daycare::TemperamentEvidence>>;

    fn append_staff_assessment(
        &self,
        assessment: daycare::NewTemperamentAssessment,
    ) -> daycare::Result<daycare::TemperamentAssessmentId>;
}

pub trait operations::daycare::EligibilityRepository {
    fn current_decision(
        &self,
        pet: entities::PetId,
        location: entities::LocationId,
        care_mode: daycare::CareMode,
    ) -> daycare::Result<Option<daycare::EligibilitySnapshot>>;

    fn append_snapshot(
        &self,
        snapshot: daycare::NewEligibilitySnapshot,
    ) -> daycare::Result<daycare::EligibilitySnapshotId>;

    fn invalidate_for_event(
        &self,
        pet: entities::PetId,
        event: daycare::EligibilityInvalidationEvent,
    ) -> daycare::Result<Vec<daycare::EligibilitySnapshotId>>;
}

impl operations::daycare::GroupPlayEligibilityPolicy {
    pub fn evaluate(
        &self,
        evidence: daycare::EligibilityEvidence,
    ) -> daycare::GroupPlayEligibilityDecision;
}

impl operations::daycare::CareModeReadinessPolicy {
    pub fn evaluate(
        &self,
        care_mode: daycare::CareMode,
        evidence: daycare::EligibilityEvidence,
    ) -> daycare::CareModeReadinessDecision;
}

impl operations::daycare::StaffCoveragePolicy {
    pub fn evaluate(
        &self,
        roster: daycare::RosterSnapshot,
        scheduled_staff: daycare::StaffCoverageSnapshot,
        ratio: daycare::StaffPetRatio,
    ) -> daycare::StaffCoverageDecision;
}

impl operations::daycare::EligibilityService {
    pub fn evaluate_request(
        &self,
        request: daycare::ReservationRequest,
    ) -> daycare::Result<daycare::EligibilityOutcome>;

    pub fn record_temperament_assessment(
        &self,
        assessment: daycare::NewTemperamentAssessment,
    ) -> daycare::Result<daycare::EligibilitySnapshotId>;

    pub fn suspend_group_play_for_incident(
        &self,
        incident: daycare::IncidentRestriction,
    ) -> daycare::Result<daycare::EligibilitySnapshotId>;

    pub fn request_reinstatement_review(
        &self,
        pet: entities::PetId,
        incident: daycare::IncidentId,
        reviewer: entities::StaffId,
    ) -> daycare::Result<workflow::task::TaskId>;
}
```

Contract rules:

- `EligibilityEvidence::builder()` should require pet, customer/location, requested service/care mode, temperament evidence state, vaccine status, age state, incident restrictions, care-note review state, policy refs, and staff/capacity snapshot. Required unknowns become typed review facts, not absent values.
- `GroupPlayEligibilityPolicy::evaluate` returns `Eligible` only when all hard requirements are satisfied and staff/capacity evidence is sufficient. It never returns eligible from missing/stale/unknown evidence.
- `CareModeReadinessPolicy` is the truthful owner for non-group paths. It may return ready for individual day boarding even when group play is denied, if room/care capacity and safety facts are satisfied.
- `EligibilityService` orchestrates repositories and policies but does not hide policy decisions in generic helpers.
- Repository methods return semantic values and `operations::daycare::error::Error`; storage adapters perform provider-code translation at the boundary.

## 5. Review / approval contract

### Automation level

Safe to automate:

- Classify service intent and determine whether the requested service could require group-play eligibility.
- Read existing pet, temperament, vaccine, incident, care-note, reservation, and roster facts.
- Build `EligibilityEvidence` and run deterministic policies.
- Produce internal readiness summaries and typed missing-requirement lists.
- Draft internal tasks for playgroup assessment, document review, incident follow-up, manager review, or customer follow-up.
- Draft customer-safe messages about missing routine documentation, clearly marked for review.

Draft/recommend only:

- Suggested care-mode routing when group play is not eligible but individual day boarding may be possible.
- Suggested playgroup assignment/rationale from current evidence.
- Suggested reinstatement packet after incident review facts are present.
- Customer follow-up wording for eligibility, behavior, health, or incident-sensitive topics.

Never automate without required human approval:

- Initial temperament assessment, behavior interpretation, final playgroup assignment, or clearing a behavior review gate.
- Manager override of spay/neuter, vaccine, incident, staff ratio, capacity, or age requirements.
- Suspension or reinstatement of group-play eligibility after an incident.
- Sending incident, health, safety, or sensitive behavior messages to customers.
- Confirming a group-play reservation or changing a reservation when eligibility/capacity is unresolved.
- Payment, discount, refund, membership, or package changes.

### Review gates

- `policy::ReviewGate::BehaviorReview`: initial/stale temperament assessment, behavior flags, group assignment from observed behavior, clearing `NeedsStaffReview`.
- `policy::ReviewGate::MedicalDocumentReview`: vaccine, spay/neuter document ambiguity, care-note/medical handling uncertainty.
- `policy::ReviewGate::ManagerApproval`: overrides, suspension/reinstatement, hard-stop exceptions, staff/capacity exceptions.
- `policy::ReviewGate::CustomerMessageApproval`: any customer-facing eligibility, behavior, health, incident, or sensitive-care message.

### Audit trail

Each eligibility snapshot and review action should record:

- pet, customer, location, requested service/care mode, and optional reservation id;
- policy ids/versions used for evaluation;
- evidence ids and evidence freshness;
- decision/reasons/review gates;
- actor/source: system policy, staff assessment, manager review, customer document, or adapter import;
- created/replaced snapshot id and timestamp;
- review/approval actor and rationale for overrides or reinstatement;
- customer-message draft id and approval status if communication is generated.

Customer/member-facing boundaries:

- The system may show staff a readiness state and a draft explanation.
- The system must not send sensitive behavior/incident/health conclusions to the customer without the configured approval.
- The system must not promise group play, availability, assignment, or pricing until deterministic policy and staff/manager gates are satisfied.

## 6. Test contracts

Later Rust implementation should include semantic tests like:

1. `missing_temperament_assessment_routes_group_play_to_staff_review`
   - All-day/half-day group play with no current assessment returns `NeedsStaffReview`, creates/suggests `PlaygroupAssessment`, and does not assign `CareMode::DogGroupPlay`.

2. `stale_temperament_assessment_is_review_not_eligibility`
   - Evidence beyond the location freshness window returns `EligibilityReviewReason::StaleTemperamentAssessment`.

3. `comfortable_group_observation_alone_does_not_bypass_vaccine_or_spay_neuter_policy`
   - Positive temperament evidence cannot produce `Eligible` when required vaccine or spay/neuter facts are unknown/unsatisfied.

4. `bite_history_or_manager_review_flag_suspends_or_denies_group_play`
   - `temperament::BehaviorObservation::BiteHistory` or `RequiresManagerReview` produces a review/denial/suspension outcome with manager/behavior gate.

5. `unknown_source_facts_never_default_to_group_play_eligible`
   - Unknown temperament, vaccine, spay/neuter, incident, staff, or policy facts return typed review reasons.

6. `day_boarding_can_remain_available_when_group_play_is_denied`
   - Group-play denial does not block individual day boarding unless individual-care safety/capacity rules fail.

7. `cat_individual_playtime_uses_cat_enrichment_readiness_not_dog_group_play_policy`
   - Cat daycare never requires dog playgroup assignment or dog spay/neuter group-play policy by default.

8. `day_play_plus_room_requires_both_play_readiness_and_room_readiness`
   - Hybrid care requires both applicable play/enrichment readiness and dedicated room/rest capacity.

9. `incident_restriction_invalidates_existing_eligible_snapshot`
   - A suspending incident appends a new snapshot and makes the current group-play state `TemporarilySuspended` until manager review.

10. `manager_reinstatement_appends_new_snapshot_instead_of_mutating_history`
    - Reinstatement records an approval/audit event and new eligibility snapshot while preserving the old suspension snapshot.

11. `staff_coverage_unknown_or_insufficient_blocks_playgroup_assignment`
    - `StaffCoverageDecision::Unknown` or `Insufficient` prevents `PlaygroupAssignment` and routes to staff/manager review.

12. `front_desk_ready_requires_resolved_eligibility_care_capacity_and_documents`
    - Check-in readiness is ready only when eligibility, care notes, documents, staff/capacity, and assignment are resolved.

13. `eligibility_snapshot_redacts_sensitive_staff_notes_in_debug_output`
    - Debug output for snapshots/rationales does not include raw staff notes or sensitive behavior details.

14. `storage_rejects_unknown_or_zero_eligibility_values_without_silent_defaults`
    - Provider-code decoding rejects invalid zero counts, unknown required enums, and raw statuses that cannot be mapped safely.

15. `customer_message_for_behavior_or_incident_requires_customer_message_review_gate`
    - Drafts about behavior/incident/health include `ReviewGate::CustomerMessageApproval` and are not sent by policy evaluation.

16. `public_daycare_paths_preserve_semantic_context`
    - API examples use `operations::daycare::{EligibilitySnapshot, GroupPlayEligibilityDecision, TemperamentEvidence}` instead of top-level strings, booleans, or generic helper functions.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`: refine `operations::daycare` with service variants, evidence, decisions, repositories, policies, snapshots, errors, and methods. Consider splitting into `domain/src/operations/daycare/*.rs` if `operations.rs` becomes too large.
- `domain/src/temperament.rs`: add or refine staff assessment record/value types if temperament remains a neighbor context; keep raw staff notes redacted.
- `domain/src/entities.rs`: ensure `TemperamentProfile` can remain a stable pet profile while assessment records/snapshots live in daycare/temperament stores.
- `domain/src/policy.rs`: reuse `ReviewGate`, policy ids, and existing conservative play policy where appropriate; avoid duplicating global review vocabulary.
- `domain/src/workflow.rs`: add typed workflow events or task targets for daycare eligibility review if current generic task types are insufficient.
- `domain/src/agents.rs`: add or refine agent specs/default review gates for booking triage, incident escalation, manager daily brief, and daily care update.
- `domain/src/tools.rs`: ensure tool contracts distinguish read/draft/task creation from reservation confirmation, messaging send, and payment actions.
- `domain/tests/petsuites_core_service_contracts.rs`: add contract-level daycare eligibility tests.
- `domain/tests/domain_quality_patterns.rs`: add semantic quality/redaction/path tests for eligibility snapshots and staff-note handling.
- Future storage/adapters/migrations: add provider-code conversion tables for service variants, assessment ids, decision reasons, and audit events.

### Migration and refactor risks

- Existing top-level `operations::DaycareFormat` and `operations::DaycareEligibilityRule` may need re-export or migration into `operations::daycare::{ServiceVariant, EligibilityRequirement}` without breaking current tests.
- Current `policy::ConservativePlayEligibilityPolicy` returns a broad `PlayEligibilityDecision`; it should either become a compatibility adapter or be superseded by daycare-owned evidence-based `GroupPlayEligibilityPolicy`.
- `temperament::GroupPlayObservation::ComfortableInObservedGroup` is useful evidence but must not by itself mean currently eligible.
- Raw source-system service codes may be inconsistent across locations. Boundary adapters should map them into `ServiceVariant` or fail closed.
- Staff notes and behavior labels can contain sensitive data; debug/serialization/audit surfaces must preserve redaction boundaries.
- Assignment and eligibility are different concepts. Avoid merging `PlaygroupAssignment` with `GroupPlayEligibilityDecision`.
- Review gates should compose with existing `policy::ReviewGate`; do not invent parallel approval booleans.
- Code-card implementation should prefer semantic modules/re-exports over growing one flat `operations.rs` junk drawer.

### Dependencies on other implications

- Capacity/staff-ratio implications: needed for `StaffCoverageDecision` and assignment readiness.
- Incident/escalation implications: needed for suspension, reinstatement, and customer-message approval semantics.
- Daily recurring attendance implications: eligibility snapshots must be date/care-mode aware and invalidated by new evidence.
- Package/membership implications: eligibility can inform recommendations, but package actions remain separate approval-gated commercial workflows.
- Front-desk throughput/check-in implications: consumes eligibility/readiness; should not own temperament policy.
- Serialized Rust code card should implement this implication before automating reservation confirmation for group play.

Doc-only status: this artifact defines contracts and assumptions for future implementation. No live/member-facing action, policy override, storage migration, or customer communication is performed by this card.
