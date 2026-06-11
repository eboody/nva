# Daycare implication 05: Pet health/behavior notes

Purpose: model pet health and behavior notes as first-class daycare operational evidence. Notes are not free text that agents can interpret casually; they are typed, reviewable facts that affect check-in readiness, playgroup eligibility, care handling, staff tasks, and customer-safe communication.

Scope assumption: a daycare note may originate from staff observation, customer intake, uploaded document, incident follow-up, reservation handoff, or imported source-system memo. Unknown or ambiguous health/behavior language should create review work, not an eligibility clearance, customer diagnosis, or autonomous policy override.

## 1. Operational story

### Trigger

A daycare workflow receives or needs a pet-specific health/behavior note when:

- A customer requests all-day play, half-day play, day boarding, Day Play Plus Room, or cat individual playtime and provides special handling, anxiety, aggression, medication, allergy, illness, injury, or temperament details.
- Staff records an observation during check-in, playgroup assessment, daily care, incident follow-up, or checkout.
- An agent reads existing `care::*`, `temperament::*`, incident, vaccine, or reservation data and detects an unresolved note that could affect safe daycare handling.
- A current note expires, conflicts with newer evidence, or becomes relevant to a new care mode.

### Actors

- Front desk staff: collect customer-provided context, route missing/ambiguous details, and prevent fast check-in when required review is incomplete.
- Daycare attendants / playgroup staff: record observed behavior and health/care handling details; confirm which notes are operationally actionable.
- Shift lead / manager: approve restrictive dispositions, customer-facing health/behavior messages, group-play suspension/reinstatement, and overrides.
- Customer / member: provides context and receives approved summaries or follow-up requests, never raw internal speculation.
- Agent workflows: `booking-triage`, `daily-care-update`, `incident-escalation`, and `manager-daily-brief` can summarize, classify, draft, and create internal tasks within review gates.

### Inputs

- `entities::PetId`, `entities::CustomerId`, `entities::LocationId`, optional `entities::ReservationId`.
- Requested `operations::daycare::ServiceVariant` / current `CareMode`.
- Current `entities::CareProfile`, `care::MedicalNote`, `care::MedicationReviewRequirement`, `care::AllergyName`, `care::FeedingInstruction`, and medication facts.
- Current `entities::TemperamentProfile`, `temperament::GroupPlayObservation`, `temperament::TemperamentRating`, `temperament::BehaviorObservation`, and `temperament::StaffNote`.
- Existing unresolved incidents/restrictions and current `operations::daycare::GroupPlayEligibilityDecision`.
- Staff-entered note text, source metadata, timestamps, author/staff role, and customer-visible intent when known.

### Decisions

- Is the note health, behavior, mixed health/behavior, incident-related, or customer preference only?
- Is the note merely informational, operationally actionable, restrictive, or emergency/manager escalation?
- Which daycare care modes are affected: dog group play, dog individual day boarding, dog hybrid play plus room, or cat individual enrichment?
- Does the note create a review gate before check-in, group-play assignment, daily update draft, or customer message send?
- Does the note invalidate current eligibility/assignment evidence or only add handling context?
- Is the note safe for customer-facing summaries as written, or does it need staff/manager-approved wording?

### Outputs

- A typed `operations::daycare::PetNote` / `PetNoteRecord` with source, classification, visibility, affected care modes, review state, and audit metadata.
- A `PetNoteImpactDecision` that can feed front-desk readiness, eligibility, assignment, staff tasks, care watchlists, and manager briefs.
- Optional `operations::StaffTask` values: `PlaygroupAssessment`, `IncidentFollowUp`, `DocumentReview`, `DailyUpdateDraft`, `CustomerFollowUp`, or a future `DaycareNoteReview` task kind.
- Optional `OperationsRisk::PetSafetyOrCareRisk` / `DailyBriefSection::PetCareWatchlist` entries.
- Customer-message drafts only when marked draft/review-required, with sensitive/raw internals redacted.

### Success state

The note is stored as semantic daycare evidence; its operational impact is explicit; required staff/manager review gates are attached; current eligibility/readiness/assignment state is invalidated when appropriate; and any customer-facing communication remains a draft until the configured approval gate is satisfied.

### Failure and exception states

- Ambiguous note language: route to `NeedsReview { reason: AmbiguousMeaning }`; do not infer diagnosis or eligibility.
- Missing source/author/time: store as provisional evidence if useful, but require staff review before it affects clearance.
- Conflicting notes: create a conflict decision and staff/manager task; do not choose the less restrictive interpretation automatically.
- Emergency/acute health signal: escalate to manager/staff task immediately; agents may summarize but must not diagnose or tell the customer what medical action to take.
- Sensitive raw text: keep redacted debug and customer-safe projection separate from internal note content.
- Source-system import contains untyped string notes: adapter may persist raw source text but must convert into typed classification/review state before domain policy branches on it.

## 2. Domain types to add or refine

Recommended public surface under `operations::daycare`:

```rust
operations::daycare::PetNote
operations::daycare::PetNoteId
operations::daycare::PetNoteRecord
operations::daycare::PetNoteText
operations::daycare::CustomerSafePetNoteSummary
operations::daycare::PetNoteSource
operations::daycare::PetNoteKind
operations::daycare::PetNoteSeverity
operations::daycare::PetNoteVisibility
operations::daycare::PetNoteReviewState
operations::daycare::PetNoteReviewReason
operations::daycare::PetNoteDisposition
operations::daycare::PetNoteImpactDecision
operations::daycare::PetNoteConflict
operations::daycare::PetNoteEvidenceSnapshot
operations::daycare::PetNotePolicy
operations::daycare::PetNoteImpactPolicy
operations::daycare::PetNoteRepository
operations::daycare::PetNoteAuditRepository
operations::daycare::PetNoteReviewService
operations::daycare::PetNoteCustomerSummaryPolicy
```

Recommended refinements to existing surfaces:

- Keep `care::*` as the owner of stable medical/care profile values (`MedicalNote`, medication, allergies, feeding, contact refs). Daycare should reference those facts as evidence, not duplicate medical truth.
- Keep `temperament::*` as the owner of observed behavior vocabulary (`StaffNote`, `BehaviorObservation`, `GroupPlayObservation`, ratings). Daycare should own how those observations affect daycare care modes.
- Refine `operations::PetCareWatchReason` with daycare-specific projections, or add `operations::daycare::PetCareWatchReason` if the reasons become specific to care-mode readiness.
- Consider adding `operations::StaffTaskKind::DaycareNoteReview { pet_id }` only if `DocumentReview`, `PlaygroupAssessment`, and `IncidentFollowUp` are not precise enough for the implementation card.

Semantic types and invariants:

```rust
operations::daycare::PetNoteId
// Non-empty provider-neutral identifier or typed UUID wrapper. Source-system memo IDs convert at adapters.

operations::daycare::PetNoteText
// Trimmed, non-empty, length-bounded, redacted Debug. Raw internal note body is never logged verbatim.

operations::daycare::CustomerSafePetNoteSummary
// Trimmed, non-empty, length-bounded, redacted Debug. Constructed only after review-safe wording rules pass.

operations::daycare::PetNoteSource::{
    CustomerIntake,
    FrontDeskEntry { staff_id: entities::StaffId },
    DaycareStaffObservation { staff_id: entities::StaffId },
    PlaygroupAssessment { staff_id: entities::StaffId },
    IncidentFollowUp { incident_id: operations::daycare::IncidentId },
    ImportedSourceSystem { system: SourceSystemName },
    AgentDraft { spec: agent::Name },
}
// Invariant: agent draft source cannot be final approval evidence by itself.

operations::daycare::PetNoteKind::{
    HealthCareHandling,
    MedicationOrAllergy,
    IllnessOrInjuryConcern,
    AnxietyOrStress,
    GroupPlayBehavior,
    HumanHandlingBehavior,
    IncidentRelated,
    CustomerPreference,
    MixedHealthAndBehavior,
}
// Invariant: kind drives review policy; free text alone never drives eligibility.

operations::daycare::PetNoteSeverity::{
    Informational,
    HandlingInstruction,
    StaffReviewRequired,
    ManagerReviewRequired,
    SuspendGroupPlayPendingReview,
    EmergencyEscalation,
}
// Invariant: restrictive severities cannot be downgraded by an agent.

operations::daycare::PetNoteVisibility::{
    InternalOnly,
    CustomerProvided,
    CustomerSafeAfterReview,
    CustomerFacingApproved,
}
// Invariant: internal-only text cannot be sent; approved summary must be a separate value.

operations::daycare::PetNoteReviewState::{
    Draft,
    NeedsStaffReview { reasons: Vec<PetNoteReviewReason> },
    NeedsManagerReview { reasons: Vec<PetNoteReviewReason> },
    ApprovedForOperations { approved_by: entities::StaffId },
    ApprovedForCustomerSummary { approved_by: entities::StaffId },
    Superseded { by: PetNoteId },
}
// Invariant: customer-facing output requires ApprovedForCustomerSummary or a workflow ReviewGate approval.

operations::daycare::PetNoteDisposition::{
    NoOperationalRestriction,
    AddHandlingInstruction,
    RequireCareReviewBeforeCheckIn,
    RequirePlaygroupAssessment,
    RestrictToIndividualCare,
    SuspendGroupPlayPendingManagerReview,
    EscalateIncidentOrHealthConcern,
}
// Invariant: disposition is the operational result, not a raw note label.
```

`PetNoteRecord` should be built, not assembled from public fields:

```rust
pub struct PetNoteRecord {
    pub id: PetNoteId,
    pub pet_id: entities::PetId,
    pub customer_id: Option<entities::CustomerId>,
    pub reservation_id: Option<entities::ReservationId>,
    pub location_id: entities::LocationId,
    pub source: PetNoteSource,
    pub kind: PetNoteKind,
    pub severity: PetNoteSeverity,
    pub text: PetNoteText,
    pub visibility: PetNoteVisibility,
    pub review_state: PetNoteReviewState,
    pub affected_care_modes: Vec<CareMode>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

Builder invariants:

- `pet_id`, `location_id`, `source`, `kind`, `severity`, `text`, and `created_at` are required.
- `affected_care_modes` is non-empty for any note that affects daycare eligibility, assignment, check-in, or care handling.
- `AgentDraft` source must start in `Draft` or `NeedsStaffReview`, never approved.
- `EmergencyEscalation`, `SuspendGroupPlayPendingReview`, and unresolved incident-related notes must produce manager/safety review states.
- Raw text and customer-safe summary are different values; the model must not reuse internal notes as customer messages.

## 3. Relationship map

Entities:

- `entities::PetId`: the note is per pet; a multi-pet customer note must be split or linked explicitly.
- `entities::CustomerId`: optional owner/customer context for follow-up and customer-safe drafts.
- `entities::ReservationId`: optional reservation/daycare visit context; absence means the note belongs to profile-level evidence.
- `entities::StaffId`: staff author/reviewer identity for source and audit trail.
- `entities::LocationId`: scopes policy, wording, escalation, and local handling rules.

Value objects:

- `PetNoteText`, `CustomerSafePetNoteSummary`, `PetNoteId`, `SourceSystemName`, `PetNoteEvidenceSnapshotId`.
- Existing `care::*` values for medical/care content and `temperament::*` values for behavior observations.
- Existing `operations::OperationalObservation` for manager-brief projections, but not as the domain source of truth.

Policies:

- `PetNotePolicy`: validates note construction and source/review constraints.
- `PetNoteImpactPolicy`: turns note evidence into dispositions/readiness/eligibility invalidations.
- `GroupPlayEligibilityPolicy`: consumes behavior/incident/note evidence but remains the owner of group-play eligibility decisions.
- `FrontDeskThroughputPolicy`: consumes `PetNoteImpactDecision` to block/allow fast check-in.
- `PetNoteCustomerSummaryPolicy`: produces or rejects customer-safe summaries.

Repositories/stores:

- `PetNoteRepository`: append/read active note records by pet, reservation, location, and care mode.
- `PetNoteAuditRepository`: append review, approval, supersession, customer-summary, and outbound-message audit events.
- Existing/future `IncidentRepository`, `EligibilityRepository`, and `RosterRepository`: receive invalidation or review signals when notes affect incidents, eligibility, or assignment.

Workflow events:

- `DaycarePetNoteCaptured`
- `DaycarePetNoteNeedsReview`
- `DaycarePetNoteApprovedForOperations`
- `DaycarePetNoteApprovedForCustomerSummary`
- `DaycarePetNoteInvalidatedEligibility`
- `DaycarePetNoteSuperseded`

Staff tasks:

- `operations::StaffTaskKind::PlaygroupAssessment { pet_id }` for behavior/group-play review.
- `MedicationAdministration { pet_id }` or a future care-review task for medication handling impacts.
- `DocumentReview { pet_id }` for vaccine/medical/source evidence ambiguity.
- `IncidentFollowUp { pet_id }` for incident-related or suspending notes.
- `DailyUpdateDraft { reservation_id }` for customer-safe daycare updates derived from approved notes.
- Future `DaycareNoteReview { pet_id }` if implementation needs a task that is not playgroup, incident, medication, or document review.

Agent specs/tools:

- `daily-care-update`: reads approved staff notes/photos and drafts customer-safe updates; cannot diagnose, hide concerns, or send autonomously.
- `incident-escalation`: summarizes incident-related notes and drafts manager/owner review packets.
- `booking-triage`: classifies missing or conflicting health/behavior note requirements before confirming readiness.
- `manager-daily-brief`: surfaces unresolved note reviews and pet-care watchlist items.
- Tools needed: `care-note-read`, future `care-note-write-draft` or `daycare-note-draft-create`, `task-create`, `draft-message`, `incident-read`, and read-only eligibility/availability tools. Any final write/approval/send tool must require a review gate.

## 4. Interaction contract

Rust-like contracts below are intentionally domain-shaped; storage adapters can use DTOs but must promote raw strings before calling these APIs.

```rust
pub trait PetNoteRepository {
    fn active_for_pet(
        &self,
        pet_id: entities::PetId,
        location_id: entities::LocationId,
    ) -> operations::daycare::Result<Vec<operations::daycare::PetNoteRecord>>;

    fn active_for_reservation(
        &self,
        reservation_id: entities::ReservationId,
    ) -> operations::daycare::Result<Vec<operations::daycare::PetNoteRecord>>;

    fn append(
        &mut self,
        note: operations::daycare::PetNoteRecord,
    ) -> operations::daycare::Result<operations::daycare::PetNoteId>;

    fn supersede(
        &mut self,
        old: operations::daycare::PetNoteId,
        new: operations::daycare::PetNoteId,
        reviewed_by: entities::StaffId,
    ) -> operations::daycare::Result<()>;
}
```

```rust
pub trait PetNoteImpactPolicy {
    fn decide(
        &self,
        note: &operations::daycare::PetNoteRecord,
        context: &operations::daycare::PetNoteEvidenceSnapshot,
    ) -> operations::daycare::PetNoteImpactDecision;
}
```

```rust
pub enum PetNoteImpactDecision {
    NoOperationalImpact,
    AddCareHandlingInstruction { care_modes: Vec<CareMode> },
    BlocksFrontDeskReady { reasons: Vec<PetNoteReviewReason> },
    InvalidatesGroupPlayEligibility { reasons: Vec<EligibilityReviewReason> },
    RequiresPlaygroupAssessment { task: operations::StaffTask },
    RequiresManagerReview { task: operations::StaffTask },
    EscalatesIncidentOrHealthConcern { task: operations::StaffTask },
}
```

Behavior ownership:

- `PetNoteRecord::builder()` owns construction invariants: required source, text, kind, severity, visibility, review state, care-mode scope, and source/review compatibility.
- `PetNoteImpactPolicy::decide()` owns classification from typed note to operational disposition. It should not mutate repositories or send messages.
- `GroupPlayEligibilityPolicy::evaluate()` owns final group-play eligibility and consumes `PetNoteImpactDecision` as evidence; note policy should not directly assign playgroups.
- `FrontDeskThroughputPolicy::check()` owns ready/not-ready check-in status and should incorporate blocking note impacts.
- `PetNoteReviewService` coordinates repository append/review/audit/task creation; it is the truthful owner for note review workflow side effects.
- `PetNoteCustomerSummaryPolicy` owns conversion from internal note and approved facts into `CustomerSafePetNoteSummary` drafts.

Suggested service contract:

```rust
pub struct PetNoteReviewService<N, A, T, P> {
    notes: N,
    audit: A,
    tasks: T,
    impact_policy: P,
}

impl<N, A, T, P> PetNoteReviewService<N, A, T, P>
where
    N: PetNoteRepository,
    A: PetNoteAuditRepository,
    T: operations::daycare::StaffTaskRepository,
    P: PetNoteImpactPolicy,
{
    pub fn capture(
        &mut self,
        draft: PetNoteDraft,
        context: PetNoteEvidenceSnapshot,
    ) -> operations::daycare::Result<PetNoteCaptureOutcome>;

    pub fn approve_for_operations(
        &mut self,
        note_id: PetNoteId,
        reviewer: entities::StaffId,
    ) -> operations::daycare::Result<PetNoteImpactDecision>;

    pub fn approve_customer_summary(
        &mut self,
        note_id: PetNoteId,
        summary: CustomerSafePetNoteSummary,
        reviewer: entities::StaffId,
    ) -> operations::daycare::Result<CustomerSummaryApproval>;
}
```

`capture` may append the note, audit the source, and create internal tasks. It must not approve customer-facing text, clear eligibility, send a customer message, or override a suspending disposition.

## 5. Review and approval contract

Automation levels:

- Safe to automate: read existing notes; classify likely note kind/severity as a draft; identify missing source facts; draft internal summaries; create internal review tasks; add manager-brief risk items.
- Draft only: customer-safe wording for health/behavior notes, daily care updates that mention concerning facts, and incident/customer follow-up packets.
- Staff review required: any newly captured health/behavior note that changes handling, readiness, or assignment; any ambiguous customer-provided health/behavior text; any source-system import with untrusted raw strings.
- Manager approval required: group-play suspension/reinstatement, overriding restrictive dispositions, incident-related owner messages, acute health/safety escalations, and conflict resolution between notes.
- Never automate: medical diagnosis, behavioral diagnosis, hiding concerning facts, sending sensitive customer messages without approval, or converting internal notes directly into customer-facing claims.

Review gates:

- `policy::ReviewGate::BehaviorReview`: behavior notes affecting group play, temperament, human handling, anxiety/stress, or aggression indicators.
- `policy::ReviewGate::MedicalDocumentReview`: health/medical/allergy/medication notes whose source or meaning is ambiguous.
- `policy::ReviewGate::ManagerApproval`: suspensions, reinstatements, incident-related dispositions, override decisions, emergency escalations.
- `policy::ReviewGate::CustomerMessageApproval`: any customer-facing summary or owner follow-up message.

Audit trail:

- Capture: note id, source, author, location, reservation if any, raw source reference, created timestamp.
- Classification: note kind/severity, affected care modes, policy version/id, agent spec if drafted.
- Review: reviewer id, gate satisfied, decision, rationale, timestamp.
- Operational impact: readiness/eligibility invalidated, tasks created, incidents linked, old notes superseded.
- Customer boundary: approved summary id/text hash, approving staff/manager, outbound message draft id, send approval gate.

Customer/member-facing boundaries:

- Internal `PetNoteText` and `temperament::StaffNote` are not customer messages.
- `CustomerSafePetNoteSummary` must avoid diagnosis, blame, unsupported labels, and operational shorthand.
- Incident, health concern, safety, or behavior restriction messages require manager/customer-message approval even if an agent drafted them well.
- Customers can be asked for missing facts or documentation from a drafted template, but the system must not promise eligibility, availability, treatment, or policy exceptions based on unreviewed notes.

## 6. Test contracts

Named semantic tests for later implementation:

1. `daycare_pet_note_text_is_trimmed_non_empty_and_redacted_in_debug`
   - `PetNoteText::try_new("  anxious at gate  ")` stores trimmed content and debug output does not leak raw note text.

2. `agent_drafted_pet_note_cannot_start_as_approved_evidence`
   - A `PetNoteSource::AgentDraft` note built with `ApprovedForOperations` or `ApprovedForCustomerSummary` is rejected.

3. `ambiguous_health_behavior_note_blocks_front_desk_ready_until_staff_review`
   - Mixed/ambiguous note evidence produces `PetNoteImpactDecision::BlocksFrontDeskReady` and a review task, not `FrontDeskReadiness::Ready`.

4. `behavior_note_requiring_review_invalidates_group_play_without_blocking_individual_day_boarding`
   - A behavior note can invalidate dog group-play eligibility while still allowing individual day boarding review/capacity paths.

5. `health_or_medication_note_routes_to_medical_or_care_review_not_behavior_clearance`
   - Medication/allergy/illness notes use care/medical review gates and do not clear or deny group-play behavior eligibility by themselves.

6. `suspending_incident_related_note_requires_manager_approval`
   - Incident-related notes with `SuspendGroupPlayPendingReview` severity create manager review and cannot be cleared by staff-only or agent-only approval.

7. `conflicting_pet_notes_create_conflict_decision_instead_of_silent_overwrite`
   - Contradictory active notes produce a `PetNoteConflict`/manager or staff task and do not pick the less restrictive note automatically.

8. `customer_safe_summary_is_separate_from_internal_note_text`
   - Customer-facing draft APIs accept `CustomerSafePetNoteSummary`, not `PetNoteText`, and require `CustomerMessageApproval`.

9. `approved_note_capture_appends_audit_events_for_source_review_and_impact`
   - Capture and approval write audit events with source, reviewer, policy, impact, and timestamps.

10. `source_system_raw_note_is_promoted_before_policy_branching`
    - Imported raw notes cannot drive eligibility/readiness until converted into `PetNoteKind`, `PetNoteSeverity`, `ReviewState`, and affected care modes.

11. `emergency_health_signal_escalates_without_customer_diagnosis`
    - Emergency severity creates manager/staff escalation and forbids autonomous diagnosis or owner-message send.

12. `daycare_daily_update_draft_uses_only_approved_or_review_safe_note_summaries`
    - `daily-care-update` can draft from approved summaries or staff-reviewed note projections, not raw unreviewed internal text.

13. `pet_note_impact_policy_uses_care_mode_scope`
    - A note scoped to `CatIndividualEnrichment` does not accidentally block dog group-play policy, and a dog group-play note does not force cat rules.

14. `superseded_pet_note_no_longer_blocks_current_readiness_but_remains_auditable`
    - Supersession removes the note from active readiness decisions while preserving audit/history.

## 7. Integration notes for the later serialized Rust code card

Likely files touched:

- `domain/src/operations.rs`: add `operations::daycare` note types, note impact decisions, repository/service traits, and possibly daycare-specific staff task/reason enums.
- `domain/src/care.rs`: reuse/redact medical/care note values; add small review/value refinements only if daycare note contracts expose missing stable care concepts.
- `domain/src/temperament.rs`: reuse/redact behavior observations/staff notes; add explicit behavior evidence variants only when required by daycare decisions.
- `domain/src/policy.rs`: ensure `ReviewGate` and `AutomationLevel` cover behavior, medical document/care review, manager approval, and customer-message approval. Avoid making policy own daycare note semantics.
- `domain/src/agents.rs`: refine baseline specs/tool names if daycare-note drafting/review tools become explicit.
- `domain/src/tools.rs`: add or refine tool boundary names for note draft/read/audit/task creation if needed.
- `domain/tests/domain_quality_patterns.rs`: add redaction, builder, and review-gate tests.
- `domain/tests/petsuites_core_service_contracts.rs`: add daycare note/impact policy contract tests or split into a dedicated `daycare_pet_notes.rs` test file.

Migration/refactor risks:

- Current `care::MedicalNote`, `temperament::StaffNote`, and `OperationalObservation` are useful but too raw/generic to own daycare operational disposition. Do not branch on their text directly.
- Current `operations::PetCareWatchReason::{AnxietyOrStressFlag, BehaviorReview, IncidentFollowUp}` can project note impacts into briefs, but should not become the source of note truth.
- Raw source-system notes may contain sensitive or customer-facing-inappropriate language. Preserve source references for audit, but promote into semantic values before policy decisions.
- Adding `StaffTaskKind::DaycareNoteReview` may be clearer, but avoid task-kind sprawl if existing `PlaygroupAssessment`, `DocumentReview`, and `IncidentFollowUp` express the truthful workflow.
- Customer-safe summary modeling must not accidentally expose internal `Debug` values or reuse staff note text.

Dependencies on other daycare implications:

- Group-play eligibility and assignment: pet note impacts feed `GroupPlayEligibilityPolicy` and `PlaygroupAssignment`; they should not replace those owners.
- Front-desk readiness: note review state is a readiness input; unresolved health/behavior notes block fast check-in.
- Incident policy: incident-related notes can create or link to incident follow-up and suspending dispositions.
- Daily care updates/Pawgress-style reports: customer summaries depend on approved note projections.
- Package/membership/revenue opportunities: note restrictions may affect recommendation wording, but commercial suggestions must not obscure safety/care facts.

Doc-only status: this artifact models domain contracts and operational implications. It does not change code, schemas, live systems, customer/member data, or operational policy.
