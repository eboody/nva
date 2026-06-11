# Boarding implication 05: Pet profile requirements

Purpose: define the Boarding operational implication for pet-profile readiness before a boarding stay moves from request/pre-arrival into staff-operable check-in. This is a modeling/spec artifact for later Rust cards. It does not authorize live reservation changes, customer messages, medical approval, vaccine approval, payment actions, or member-facing automation.

Assumptions:

- Boarding consumes the canonical pet master record from `entities::Pet`; it must not create a parallel boarding-only pet record that can drift from customer/pet truth.
- PetSuites locations can vary in required vaccines, document freshness windows, medication-handling rules, and special-handling review policy. The domain should encode a conservative default plus location-scoped policy refs.
- Unknown or conflicting pet profile facts are operational blockers/review gates, not values to infer from prior stays unless a typed policy permits staff-reviewed reuse.
- Dogs and cats share the profile-readiness workflow, but species-specific vaccine, play, potty-walk, housing, and care-review rules must stay typed.

## 1. Operational story

### Trigger

A Boarding pet-profile requirement evaluation starts when one of these events occurs:

- a customer or staff member requests a boarding stay;
- an existing boarding reservation enters pre-arrival/check-in preparation;
- a customer uploads vaccine/medical/care documents;
- staff edits feeding, medication, temperament, special-handling, or emergency-contact facts;
- an agent notices an upcoming arrival with incomplete profile evidence.

The workflow should be usable both before confirmation and at check-in. Before confirmation it determines whether a stay can progress, should be waitlisted/missing-info, or needs staff/manager review. At check-in it determines whether staff can safely accept the pet for boarding under the current location policy.

### Actors

- Customer/member: provides pet identity, species, profile fields, care instructions, and documents.
- Front desk / customer-care staff: reviews profile completeness, requests missing information, and records staff observations.
- Pet care staff / kennel team: verifies care-handling instructions, feeding, medication, temperament, and special-handling obligations.
- Shift lead / manager: resolves ambiguous, conflicting, expired, safety-sensitive, or exception-prone profile evidence.
- Veterinarian or licensed provider: external source of vaccine/medical documents where policy requires licensed evidence.
- AI/workflow agent: reads profile/reservation/document evidence, drafts internal tasks, summarizes missing facts, and drafts customer-message copy subject to review.

### Inputs

- `entities::Pet` with `entities::PetId`, `CustomerId`, `pet::Name`, `Species`, `birth_date`, `Sex`, `SpayNeuterStatus`, `TemperamentProfile`, and `CareProfile`.
- `entities::Reservation` / Boarding stay request with service line, date range, location, pet IDs, requested accommodation, add-ons, and status.
- `entities::Location` / `LocationPolicyRefs` for vaccine, deposit, playgroup, and local boarding policies.
- `policy::VaccineRequirement` and vaccine/document evidence from provider adapters or document tools.
- `care::FeedingInstruction`, `MedicationInstruction`, allergies, medical conditions, emergency contact, and veterinarian contact.
- `temperament::GroupPlayObservation`, behavior observations, staff notes, and previous incident/follow-up signals.
- Boarding contract facts from `operations::boarding::Contract`: accommodation, care requirements, handoff requirement, housekeeping/report obligations, and play/add-on policy.
- Existing review gates, staff tasks, workflow events, and uploaded document metadata.

### Decisions

The pet-profile requirement evaluator answers typed questions, not one broad `complete: bool`:

1. Is the pet identified well enough for Boarding? Name, customer ownership, species, and reservation linkage must be present and semantically valid.
2. Is the species compatible with the requested accommodation and add-ons?
3. Are location/service-specific vaccine requirements satisfied by acceptable evidence?
4. Are feeding instructions present or explicitly unnecessary under local policy?
5. Are medication instructions absent, complete, or review-required?
6. Are allergies/medical conditions/emergency/veterinarian contacts sufficient for an overnight stay?
7. Is temperament/play eligibility clear enough for requested enrichment, group play, potty walks, or special handling?
8. Are conflicting facts present across customer input, staff notes, prior records, and uploaded documents?
9. Which missing or ambiguous facts can be handled by staff review, and which require manager/vet/licensed-document review?
10. What internal tasks, review gates, reservation status suggestions, and audit events must be produced?

### Outputs

- `operations::boarding::profile::ReadinessDecision`:
  - `Ready { summary, evidence }`;
  - `ReadyWithStaffInstructions { care_plan, tasks }`;
  - `MissingInformation { missing, tasks }`;
  - `StaffReviewRequired { gates, tasks }`;
  - `ManagerReviewRequired { gates, reasons, tasks }`;
  - `CannotProceed { reasons, recommended_status }`.
- A `boarding::care::Plan` or `profile::CareReadiness` projection for check-in/pre-arrival tasks.
- Internal `operations::StaffTask` drafts for missing profile fields, document review, care clarification, medication double-check, behavior review, emergency-contact completion, and accommodation/play mismatch review.
- `workflow::RecommendedAction` values such as `InternalTask`, `RequestHumanReview`, or draft-only customer-message recommendations.
- `workflow::WorkflowEvent` audit entries for profile requirement evaluation, evidence reviewed, review gates raised, and staff/manager approvals.
- Optional draft customer message copy when missing information needs outreach; send remains review-gated.

### Success state

A boarding stay is profile-ready when every required pet-profile dimension has one of these typed outcomes:

- satisfied by valid canonical pet/customer/reservation data;
- satisfied by current acceptable document evidence;
- explicitly not required by the location/service policy;
- accepted by a recorded staff or manager approval matching the review gate;
- converted into a concrete staff care instruction/task that can be fulfilled during the stay.

The success state is not “all fields are non-null.” It is “the boarding team has enough verified, policy-compatible, auditable pet profile facts to safely house and care for this pet for this stay.”

### Failure and exception states

- Missing required identity/profile fields: no species, no customer linkage, unknown pet identity, or no usable pet profile.
- Missing or expired vaccine evidence for the species/service/location.
- Medication present without dose/schedule/review requirement, or conflict between customer instructions and staff notes.
- Feeding instructions absent when policy requires an explicit overnight feeding plan.
- Allergies/medical conditions present without sufficient handling instructions.
- Emergency/veterinarian contact missing when overnight/special-care policy requires it.
- Temperament/group-play evidence ambiguous, negative, stale, or incompatible with requested add-on/accommodation.
- Requested dog/cat accommodation or playtime mismatches species.
- Conflicting profile facts across sources: customer portal, staff notes, document OCR, previous reservation, or imported provider data.
- Review gate exists but lacks the right actor approval.
- Agent attempted to convert a profile finding directly into a customer-facing message, reservation confirmation, profile overwrite, or care/medical approval.

## 2. Domain types to add or refine

### Canonical pet-profile readiness module

Prefer a Boarding-owned projection that evaluates canonical pet data rather than duplicating master pet state:

- `operations::boarding::profile::RequirementSet`
  - Location/service/accommodation-specific set of pet-profile requirements for one stay.
  - Invariants: non-empty when Boarding contract requires profile review; all requirements have typed source policy or contract reason.
- `operations::boarding::profile::Requirement`
  - Enum: `Identity`, `SpeciesAccommodationCompatibility`, `Vaccination(VaccineRequirementRef)`, `FeedingInstruction`, `MedicationInstruction`, `AllergyAndMedicalConditionReview`, `EmergencyContact`, `VeterinarianContact`, `TemperamentReview`, `PlayEligibility`, `SpecialHandling`, `AgreementOrDocument(ExpectedDocumentKind)`.
- `operations::boarding::profile::RequirementStatus`
  - Enum: `Satisfied { evidence }`, `NotRequired { policy }`, `Missing { field }`, `Conflicting { facts }`, `Expired { evidence }`, `ReviewRequired { gate }`, `Denied { reason }`.
- `operations::boarding::profile::ReadinessDecision`
  - The readiness aggregate described in the operational outputs.
  - Invariant: no `Ready` variant may contain missing/conflicting/expired/denied statuses.
- `operations::boarding::profile::ReadinessSummary`
  - Staff-readable bounded summary for shift handoff and daily brief; non-empty, redaction-aware.
- `operations::boarding::profile::EvidenceRef`
  - Typed reference to canonical profile field, staff note, uploaded document, policy snapshot, prior approved review, or provider record.
  - Invariant: must carry source kind, captured/reviewed timestamp where known, actor/source, and trust boundary.
- `operations::boarding::profile::StalenessPolicy`
  - Decides whether prior evidence can satisfy current stay requirements.
- `operations::boarding::profile::Conflict`
  - Typed conflict between profile facts. Carries conflicting evidence refs and domain-specific conflict kind.

### Refinements to existing root entities

- `entities::Pet`
  - Keep as master entity. Add/refine semantic fields only when required globally: `profile::Version`, `profile::UpdatedAt`, or profile source metadata if all service lines need it.
  - Do not move Boarding-specific readiness into `entities::Pet`.
- `entities::CareProfile`
  - Keep feeding, medications, allergies, medical conditions, emergency and vet contacts as canonical care facts.
  - Add explicit completeness/review methods only if service-agnostic; otherwise Boarding-specific evaluation belongs in `operations::boarding::profile::Policy` / `care::Policy`.
- `entities::TemperamentProfile`
  - Continue to own observed temperament facts. Boarding/play-specific eligibility remains in `policy::PlayEligibilityPolicy` and `operations::boarding::playtime`.
- `booking_triage::PetProfile`
  - Current model has only `name` and `PetProfileCompleteness`. Refactor or bridge it into `operations::boarding::profile::ReadinessDecision`; avoid expanding this triage helper into a parallel profile source.

### Newtypes and scalar invariants

- `operations::boarding::profile::RequirementId`
  - Non-empty opaque requirement identifier for audit/task linking.
- `operations::boarding::profile::PolicyVersion`
  - Non-empty location policy snapshot/version.
- `operations::boarding::profile::EvidenceId`
  - Non-empty external/provider/document/staff-note evidence id.
- `operations::boarding::profile::ReviewedAt`
  - Timestamp wrapper if local policy needs freshness semantics separate from `DateTime<Utc>`.
- `operations::boarding::profile::FreshnessDays`
  - Positive scalar for how long evidence remains acceptable.
- `operations::boarding::profile::DocumentLabel`
  - Non-empty bounded label for uploaded/generated documents.
- `operations::boarding::profile::StaffInstruction`
  - Non-empty bounded internal instruction; never customer-facing without message review.

### Semantic enums

- `profile::ReadinessPhase`
  - `BookingRequest`, `PreArrival`, `CheckIn`, `DuringStay`, `Checkout`; requirements may differ by phase.
- `profile::EvidenceKind`
  - `CanonicalPetField`, `CustomerProvidedAnswer`, `StaffObservation`, `UploadedDocument`, `LicensedVetDocument`, `PriorApprovedReview`, `ProviderRecord`, `PolicySnapshot`.
- `profile::MissingField`
  - `Species`, `SpayNeuterStatus`, `FeedingInstruction`, `MedicationDose`, `MedicationSchedule`, `EmergencyContact`, `VeterinarianContact`, `VaccineDocument`, `TemperamentObservation`, `SpecialHandlingInstruction`, etc.
- `profile::ConflictKind`
  - `SpeciesMismatch`, `MedicationMismatch`, `FeedingMismatch`, `VaccineRecordMismatch`, `TemperamentMismatch`, `CustomerStaffNoteMismatch`, `PolicyVersionMismatch`.
- `profile::DenialReason`
  - `RequiredVaccineMissing`, `AccommodationSpeciesMismatch`, `MedicalReviewUnresolved`, `BehaviorReviewUnresolved`, `ProfileIdentityUnverified`, `PolicyProhibitsStay`.
- `profile::ReviewGate`
  - Boarding-specific gate that maps to `policy::ReviewGate`: `StaffProfileReview`, `MedicalDocumentReview`, `MedicationDoubleCheck`, `BehaviorReview`, `ManagerException`, `CustomerMessageApproval`.

### Policies and builders

- `operations::boarding::profile::Policy`
  - Deterministic evaluator. Owns the readiness decision, not a free helper.
- `operations::boarding::profile::RequirementSet::builder()`
  - Required: location, service phase, contract version, policy version, species/accommodation basis.
  - Build should fail if no source policy/contract reason is attached.
- `operations::boarding::profile::ReadinessDecision::from_statuses(...)`
  - Constructor enforces variant/status consistency.
- `operations::boarding::care::Plan::builder()`
  - Should accept profile readiness evidence and produce staff-operable care obligations only after review gates are handled or explicitly carried forward as tasks.

## 3. Relationship map between types

### Entities

- `entities::Pet` is the master pet entity. Boarding evaluates it but does not own it.
- `entities::Customer` owns member/contact identity. Profile outreach uses `CustomerId`/preferred channel; Boarding does not store duplicate contact details.
- `entities::Reservation` owns cross-service reservation lifecycle. Boarding readiness suggests status transitions; it does not directly mutate live status.
- `entities::Location` owns brand, timezone, capabilities, and policy refs used to select requirement sets.

### Value objects

- `pet::Name`, `entities::Species`, `SpayNeuterStatus`, `care::FeedingInstruction`, `care::MedicationName/Dose/Schedule`, `care::AllergyName`, `care::MedicalConditionName`, `care::ContactRef`, and temperament values remain canonical service-agnostic facts.
- `boarding::profile::EvidenceRef`, `RequirementId`, `PolicyVersion`, `FreshnessDays`, and `StaffInstruction` are Boarding-owned operational values.
- `boarding::accommodation::Kind` and `boarding::playtime::Kind` determine which profile requirements are relevant.

### Policies

- `policy::VaccineRequirement` defines species/service vaccine obligations.
- `policy::PlayEligibilityPolicy` evaluates base play eligibility.
- `boarding::profile::Policy` composes vaccine, care, temperament, accommodation, document, and phase requirements into a readiness decision.
- `boarding::care::Policy` consumes profile readiness to produce care-plan obligations and review tasks.
- `boarding::agent::ApprovalPolicy` maps readiness outputs to automation/review gates.

### Repositories and stores

- `pet::Repository` / `entities` store: read canonical pet/customer/location/reservation facts.
- `boarding::profile::Repository`: store readiness evaluations, evidence refs, conflict records, and approval snapshots. It should not store raw duplicate profile fields.
- `boarding::care::Repository`: read care profile/task evidence; write internal care-plan/task projections.
- `boarding::reservation::Repository`: query related reservations and suggest status updates as workflow/tool drafts.
- `tools::DocumentCollection` / document provider ports: collect/read document metadata and OCR/extraction results; domain validates against semantic requirements.

### Workflow events

- Existing `workflow::WorkflowEventType::PetProfileCreated`, `VaccineDocumentUploaded`, and `BookingTriageNeeded` should be reused.
- Add later if behavior demands: `PetProfileReviewNeeded`, `PetProfileRequirementSatisfied`, `PetProfileConflictDetected`, `BoardingCheckInBlocked`.
- Workflow events carry `PolicyContext`, `AllowedAction`, `AutomationLevel`, and `ReviewGate` so audit/review status is explicit.

### Staff tasks

- `boarding::task::Kind::PetProfileReview`
- `boarding::task::Kind::VaccineDocumentReview`
- `boarding::task::Kind::FeedingInstructionClarification`
- `boarding::task::Kind::MedicationDoubleCheck`
- `boarding::task::Kind::BehaviorOrPlayEligibilityReview`
- `boarding::task::Kind::EmergencyContactCompletion`
- `boarding::task::Kind::AccommodationSpeciesMismatchReview`

These map to generic `operations::StaffTask` while preserving Boarding-specific source/reason/evidence refs.

### Agent specs/tools

- Agent specs:
  - `boarding::agent::PreArrivalProfileAuditor`
  - `boarding::agent::CheckInReadinessAssistant`
  - `boarding::agent::MissingInfoDraftAssistant`
- Tool boundaries:
  - read pet/customer/reservation/care profile;
  - read uploaded document metadata/extracted facts;
  - create internal task drafts;
  - draft customer-message copy;
  - request human review.
- Disallowed without approval:
  - overwrite pet profile facts;
  - approve vaccines/medical/behavior facts;
  - confirm/cancel/modify reservations;
  - send member-facing messages.

## 4. Interaction contract

Rust-like pseudo-signatures below name truthful owners. They are contracts for later implementation, not required exact syntax.

```rust
pub mod operations::boarding::profile {
    pub struct Policy { /* location-scoped rule set and adapters */ }

    impl Policy {
        pub fn requirement_set_for(
            &self,
            location: &entities::Location,
            contract: &operations::boarding::Contract,
            phase: ReadinessPhase,
            stay: &operations::boarding::StayRequest,
        ) -> Result<RequirementSet>;

        pub fn evaluate(
            &self,
            requirements: &RequirementSet,
            pet: &entities::Pet,
            reservation: &entities::Reservation,
            evidence: EvidenceBundle,
        ) -> Result<ReadinessDecision>;
    }

    pub struct RequirementSet { /* typed requirements */ }

    impl RequirementSet {
        pub fn requirements(&self) -> &[Requirement];
        pub fn requires(&self, requirement: RequirementKind) -> bool;
    }

    pub enum ReadinessDecision { /* Ready, MissingInformation, ReviewRequired, CannotProceed */ }

    impl ReadinessDecision {
        pub fn can_proceed_to_check_in(&self) -> bool;
        pub fn required_reviews(&self) -> Vec<policy::ReviewGate>;
        pub fn staff_tasks(&self) -> Vec<operations::boarding::task::Draft>;
        pub fn recommended_reservation_status(&self) -> Option<entities::ReservationStatus>;
        pub fn audit_summary(&self) -> ReadinessSummary;
    }

    pub trait Repository {
        fn latest_for_reservation(
            &self,
            reservation_id: entities::ReservationId,
        ) -> Result<Option<ReadinessDecision>>;

        fn save_evaluation(
            &mut self,
            reservation_id: entities::ReservationId,
            decision: ReadinessDecision,
            event: workflow::WorkflowEvent,
        ) -> Result<()>;
    }
}
```

```rust
pub mod operations::boarding::care {
    impl Policy {
        pub fn plan_from_profile_readiness(
            &self,
            pet: &entities::Pet,
            readiness: &boarding::profile::ReadinessDecision,
            requested_add_ons: &[entities::AddOn],
        ) -> Result<Plan>;
    }

    impl Plan {
        pub fn open_review_gates(&self) -> &[boarding::profile::ReviewGate];
        pub fn staff_instructions(&self) -> &[boarding::profile::StaffInstruction];
        pub fn handoff_items(&self) -> Vec<boarding::handoff::Item>;
    }
}
```

```rust
pub mod operations::boarding::workflow {
    impl Planner {
        pub fn plan_profile_requirement_work(
            &self,
            decision: &boarding::profile::ReadinessDecision,
            actor: workflow::ActorRef,
        ) -> workflow::WorkflowResult<boarding::profile::ReadinessSummary>;
    }
}
```

```rust
pub mod operations::boarding::agent {
    impl ApprovalPolicy {
        pub fn classify_profile_action(
            &self,
            action: &boarding::profile::RecommendedAction,
        ) -> policy::AutomationRule;
    }
}
```

Repository behavior:

- Profile repositories persist evaluations, evidence refs, approval snapshots, and generated task/message draft ids.
- They do not persist raw external JSON as domain values; adapters translate provider fields into semantic evidence refs and extracted facts.
- Writes that would alter canonical pet profile data must go through pet/customer/profile management boundaries with staff approval, not through Boarding readiness evaluation.

Service behavior:

- `boarding::profile::Policy` owns readiness decisions.
- `boarding::care::Policy` owns care-plan obligations after/alongside readiness.
- `boarding::workflow::Planner` owns task/action/event shaping.
- `boarding::agent::ApprovalPolicy` owns automation gates.
- No `utils::is_pet_profile_complete(...)` helper should become the semantic center.

## 5. Review and approval contract

### Automation level

Safe/internal automation:

- read canonical pet/reservation/location/care/temperament/document metadata;
- compute deterministic readiness statuses;
- summarize missing or conflicting profile requirements;
- draft internal staff tasks and handoff items;
- draft customer-message copy for staff review;
- attach audit evidence refs to a workflow result.

Staff review required:

- accepting vaccine/document evidence as satisfying policy;
- resolving missing feeding instructions or care details;
- verifying medication dose/schedule/instructions;
- approving customer-facing missing-info messages;
- marking profile readiness as satisfied when evidence is free-text, OCR-derived, or manually interpreted;
- checking in a pet whose readiness includes staff-carried instructions.

Manager review required:

- overriding missing/expired vaccine or profile requirements;
- resolving medical, behavior, incident, safety, or accommodation incompatibility issues;
- accepting conflicting customer/staff/provider facts;
- approving exceptions to local policy or stay acceptance;
- approving sensitive customer-facing explanations.

Never fully automate:

- medical/vaccine approval;
- behavior safety approval;
- profile overwrites in canonical pet records;
- reservation confirmation/cancellation/rejection based on profile status;
- customer/member-facing messages about denial, medical, behavior, vaccine, or safety matters;
- hiding negative or unresolved care facts from reports/handoffs.

### Review gates

Map Boarding-specific gates to existing `policy::ReviewGate` where possible:

- `StaffProfileReview` -> staff review task / `InternalTaskOnly`.
- `MedicalDocumentReview` -> `policy::ReviewGate::MedicalDocumentReview`.
- `MedicationDoubleCheck` -> staff review task; manager if conflict/safety risk.
- `BehaviorReview` -> `policy::ReviewGate::BehaviorReview`.
- `ManagerException` -> `policy::ReviewGate::ManagerApproval`.
- `CustomerMessageApproval` -> `policy::ReviewGate::CustomerMessageApproval`.

A review gate must carry actor role, evidence refs, reason, timestamp, and scope. Approval of one field does not approve unrelated fields or future stays unless a `StalenessPolicy` explicitly allows reuse.

### Audit trail

Every readiness evaluation should record:

- reservation id, pet id, customer id, location id;
- phase (`BookingRequest`, `PreArrival`, `CheckIn`, etc.);
- policy version/location policy refs;
- requirement statuses and evidence refs;
- generated staff tasks and workflow actions;
- review gates raised/cleared and approving actor;
- agent id/version/prompt packet if an agent generated drafts/summaries;
- whether any member-facing draft was produced, sent, or blocked.

### Customer/member-facing boundaries

- Drafting a missing-info message is allowed as `DraftOnly` with `CustomerMessageApproval`.
- Sending a message requires explicit staff/manager approval and an approved contact channel.
- Denial, vaccine, medical, behavior, incident, safety, and accommodation-incompatibility explanations are manager-review customer communications.
- Internal profile-readiness labels must not leak directly to customers; staff-approved messages should be empathetic, specific, and policy-grounded.

## 6. Test contracts

Future code cards should add semantic tests with names like these:

### Requirement-set construction

- `boarding_profile_requirement_set_includes_species_vaccine_feeding_medication_emergency_contact_and_temperament_for_overnight_stay`
  - A standard PetSuites Boarding pre-arrival requirement set contains typed requirements, not raw checklist strings.
- `boarding_profile_requirement_set_varies_by_location_policy_version_without_mutating_pet_profile`
  - Location policy changes alter requirement evaluation, not canonical pet fields.
- `boarding_profile_requirement_set_distinguishes_dog_group_play_from_cat_individual_enrichment`
  - Species-specific play/accommodation requirements remain typed.

### Readiness decisions

- `boarding_profile_ready_requires_all_required_statuses_satisfied_or_approved`
  - `Ready` cannot be constructed when any required status is missing, conflicting, expired, denied, or uncleared review.
- `boarding_profile_missing_feeding_instruction_creates_staff_task_not_silent_default`
  - Missing feeding instructions produce `MissingInformation`/task.
- `boarding_profile_medication_without_dose_or_schedule_requires_medication_double_check`
  - Medication incompleteness routes to staff/manager review.
- `boarding_profile_missing_emergency_contact_blocks_checkin_when_location_requires_it`
  - Overnight policy-required contact is enforced.
- `boarding_profile_species_accommodation_mismatch_cannot_proceed_without_manager_review`
  - Dog/cat accommodation mismatch returns typed denial/review.

### Evidence and conflicts

- `boarding_profile_vaccine_document_requires_licensed_evidence_when_policy_requires_it`
  - Untrusted customer text cannot satisfy licensed-vet requirement.
- `boarding_profile_expired_vaccine_evidence_routes_to_document_review`
  - Expired evidence does not satisfy readiness.
- `boarding_profile_conflicting_medication_instructions_preserve_both_evidence_refs`
  - Conflicts carry typed sources for audit/review.
- `boarding_profile_prior_approval_expires_under_staleness_policy`
  - Reuse of prior approval is bounded.

### Workflow, tasks, and audit

- `boarding_profile_readiness_planner_creates_internal_tasks_for_missing_documents_and_care_reviews`
  - Planner emits internal tasks, not direct profile changes.
- `boarding_profile_review_gate_records_actor_scope_evidence_and_timestamp`
  - Approvals are auditable and scoped.
- `boarding_profile_evaluation_emits_workflow_event_with_policy_context`
  - Workflow event includes allowed actions, automation level, and review gates.
- `boarding_profile_readiness_summary_feeds_handoff_without_customer_contact_data_leakage`
  - Handoff summary is internal and redaction-aware.

### Agent boundaries

- `boarding_profile_agent_can_draft_missing_info_task_but_cannot_mark_vaccine_approved`
  - Agent output stays task/draft-only.
- `boarding_profile_agent_cannot_overwrite_canonical_pet_profile`
  - Profile writes require staff-approved canonical profile boundary.
- `boarding_profile_customer_message_draft_requires_customer_message_review_gate`
  - Any outbound draft preserves review reason and contact channel.
- `boarding_profile_medical_behavior_or_denial_message_requires_manager_approval`
  - Sensitive messages are not staff-auto-send or agent-send.

### Storage and serialization

- `boarding_profile_readiness_decision_roundtrips_without_flattening_requirement_statuses_to_strings`
  - Storage preserves enum variants and evidence refs.
- `boarding_profile_repository_stores_evaluation_snapshot_without_duplicate_pet_master_fields`
  - Repository stores readiness projection, not a second pet record.
- `booking_triage_pet_profile_completeness_maps_to_boarding_readiness_without_losing_missing_field_reason`
  - Existing triage surface can bridge into the richer model.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add `operations::boarding::profile` child module or split `operations` into service-line files if the monolith becomes too large.
  - Add Boarding profile requirement/readiness types, policies, and tests close to `boarding` semantics.
- `domain/src/entities.rs`
  - Possibly add small service-agnostic profile metadata only if needed by multiple service lines; avoid stuffing Boarding readiness into `entities::Pet`.
- `domain/src/policy.rs`
  - Reuse/extend `ReviewGate`, `AutomationLevel`, `VaccineRequirement`, and `PolicyDenialReason` rather than creating disconnected equivalents.
- `domain/src/care.rs` and `domain/src/temperament.rs`
  - Reuse care/temperament fact types; add semantic values only if current types cannot express required evidence.
- `domain/src/workflow.rs`
  - Add event/action vocabulary only after current `WorkflowEventType`, `RecommendedAction`, and `PolicyContext` prove insufficient.
- `domain/src/tools.rs`
  - Reuse document expected content (`VaccineProof`, `MedicationInstructions`, `BoardingAgreement`) and tool boundaries for collection/extraction; do not make tools own domain approval.
- `domain/src/booking_triage.rs`
  - Bridge or refactor `PetProfileCompleteness` into richer Boarding readiness. Current type is too coarse for operational check-in safety.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add initial contract-level tests for profile requirements if kept with service contract tests.
- New test file candidate: `domain/tests/petsuites_boarding_profile_requirements.rs`.
- Storage adapters/tests later under `storage::operations` once readiness snapshots need persistence.

### Migration/refactor risks

- The current `booking_triage::PetProfileCompleteness` boolean-ish enum is likely too coarse. Expanding it directly may create a generic triage dumping ground; prefer Boarding-owned readiness types plus an explicit bridge.
- `entities::CareProfile` currently defaults to empty fields. Empty default should not be interpreted as profile-ready for Boarding; readiness policy must distinguish default/unknown from not-required.
- `policy::ReviewGate` is intentionally broad. Boarding-specific gates should map to it rather than forking an unconnected approval system.
- `operations.rs` is already large. Adding all profile code inline may be expedient but risks making module ownership opaque. If a later code card touches many Boarding concepts, split service-line modules while preserving public paths.
- Storage serialization must preserve typed statuses/evidence refs. Avoid persisting `Vec<String>` checklists or unstructured JSON as the core domain representation.
- Document/OCR evidence has trust-boundary risk. Extracted text can support staff review but should not automatically satisfy licensed medical/vaccine requirements.
- Customer contact info should not leak into internal handoff/profile summaries beyond typed refs and approved communication boundaries.

### Dependencies on other implications

- Depends on the Boarding service domain map for capacity/accommodation/care/report/approval boundaries.
- Interacts with accommodation/capacity implications because species/profile compatibility can block room assignment.
- Interacts with deposit/cancellation implications only through workflow status/readiness; profile readiness must not waive payment policy.
- Interacts with care/playtime/handoff implications because readiness produces care instructions and staff tasks.
- Interacts with Pawgress Report/customer communication implications because missing-info and sensitive explanations are draft/review-gated communications.

### Recommended implementation slice

1. Add `boarding::profile::Requirement`, `RequirementStatus`, `ReadinessDecision`, `EvidenceRef`, and semantic error/result types.
2. Add `boarding::profile::Policy::evaluate(...)` for the smallest deterministic slice: identity, species/accommodation, feeding, medication, emergency contact, and vaccine evidence placeholder.
3. Add tests for missing feeding, medication double-check, species/accommodation mismatch, and no `Ready` with unresolved requirements.
4. Add workflow planner mapping readiness decisions to internal tasks and review gates.
5. Add repository/storage snapshots only after the in-memory semantic model and tests are stable.
