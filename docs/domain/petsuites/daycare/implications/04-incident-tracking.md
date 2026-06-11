# Daycare operational implication 04: Incident tracking

Purpose: model daycare incidents as safety-first operational facts that can change a pet's same-day care plan, group-play eligibility, customer notice obligations, staff follow-up work, and manager review packet. This is a modeling artifact only; it does not change live policy, source-system data, calendars, customer messages, or payment state.

Scope assumptions:

- An "incident" is any staff-observed safety, behavior, health, escape, injury, near-miss, medication/care-instruction, facility, or owner-notice event that matters to daycare operations.
- Incident tracking is daycare-owned for the daytime care workflow, but medical truth remains in `care`, temperament evidence remains in `temperament`, pet/customer identity remains in `pet`/`customer`, and message delivery remains in workflow/agent boundaries.
- Unknown or incomplete incident facts should create review work and conservative eligibility restrictions, not auto-clear a pet back into group play.
- Customer-facing incident or health messages are drafts until the configured review gate is satisfied.

## 1. Operational story

### Trigger

A daycare incident starts when one of these facts appears during or adjacent to a daycare visit:

- A staff member observes behavior that affects group-play safety: bite, attempted bite, repeated mounting, guarding, escalating chase, kennel/barrier reactivity, human selectivity, stress, or inability to settle.
- A staff member observes a health/care issue: injury, lameness, vomiting/diarrhea, allergic reaction, heat stress, missed medication, food exposure, or other care-plan deviation.
- A supervision/facility event occurs: gate/door escape attempt, yard/room overcapacity, ratio breach, incompatible playgroup mix, cleaning/sanitation hazard, or near-miss.
- A customer reports a concern after pickup that must be connected to that day's attendance and staff notes.
- An agent or manager-daily-brief workflow detects unresolved incident text, a suspending incident marker, or a staff task that should have produced a disposition.

### Actors

- `operations::daycare::staff::Reporter`: staff member who observed or received the incident facts.
- `operations::daycare::Supervisor` / `operations::StaffRole::LeadStaff`: validates the immediate operational response and confirms whether group play continues, changes, or stops.
- `operations::StaffRole::Manager`: approves severe dispositions, suspensions, reinstatements, refunds/credits if any, and customer-facing incident language.
- `customer::Customer`: receives approved owner notices and may provide follow-up observations.
- `pet::Pet`: the subject whose care mode, eligibility, and notes may change.
- `agents::incident-escalation`: read/draft/recommend agent that summarizes facts, identifies missing fields, drafts manager/owner packets, and creates internal tasks; it never closes incidents, diagnoses, sends owner messages, or clears restrictions.

### Inputs

Minimum incident capture should be explicit and typed:

- `operations::daycare::IncidentReport`: location, operating day, reservation/attendance reference, pet, reporter, observed-at timestamp, source, category, severity candidate, narrative, and affected care mode.
- `operations::daycare::incident::ObservedFact`: typed event facts such as `Behavior`, `InjuryOrHealth`, `CareInstructionDeviation`, `FacilityOrSupervision`, `CustomerReportedAfterPickup`, and `NearMiss`.
- `temperament::BehaviorObservation` and `temperament::GroupPlayObservation` when behavior evidence is relevant; these are evidence, not final daycare disposition.
- `care::*` review requirements when medical/allergy/medication/feeding facts are relevant; agents may summarize but not diagnose.
- `operations::daycare::PlaygroupAssignment` / roster snapshot if the incident happened in group play.
- `operations::daycare::StaffCoverageDecision` / ratio snapshot if staffing or supervision may be implicated.
- Optional media/document references through tool/storage boundaries, not raw blobs inside the domain aggregate.

### Decisions

Incident tracking makes or requests these decisions:

1. Immediate care response: continue current care, move to individual care/rest room, isolate safely pending pickup, request manager review, or emergency/vet escalation.
2. Severity classification: note-only, owner notice, manager review, group-play suspension, emergency/vet/safety escalation.
3. Disposition: no eligibility change, temporary same-day care-mode change, suspend group play pending review, require temperament reassessment, require medical/care review, or close as documented after approval.
4. Customer communication posture: no customer message, pickup conversation note, customer-message draft requiring approval, or urgent manager-approved contact.
5. Staff task obligations: incident follow-up, care-note review, document review, playgroup reassessment, manager review, or daily-care-update draft.
6. Eligibility invalidation: whether current and future `GroupPlayEligibilityDecision` snapshots must become `TemporarilySuspended` or `NeedsStaffReview` until a typed review clears them.

### Outputs

- `operations::daycare::Incident`: durable aggregate with report facts, severity, status, review gates, audit events, and disposition history.
- `operations::daycare::IncidentDisposition`: typed operational outcome that can drive eligibility invalidation and staff tasks.
- `operations::daycare::IncidentRestriction`: typed restriction on group play, care mode, attendance, or handling instructions.
- `operations::StaffTaskKind::IncidentFollowUp` and related staff tasks with due dates, assignments, and source references.
- `workflow::WorkflowEvent` values for incident submitted, severity classified, manager review requested, customer notice drafted, restriction applied, restriction cleared, and incident closed.
- `operations::OperationsRisk::PetSafetyOrCareRisk` and `DailyBriefSection::PetCareWatchlist` entries for manager daily briefs.
- Customer-message drafts with `policy::ReviewGate::CustomerMessageApproval` and manager/safety gates where required.

### Success state

A successful incident workflow has:

- The incident report is complete enough to explain who/what/when/where/source, which pet/reservation was involved, what immediate action was taken, and which facts are still unknown.
- Severity and disposition are typed and conservative; unresolved hard stops cannot silently disappear.
- Required review gates and staff tasks exist and point back to the incident.
- Group-play eligibility snapshots are invalidated or restricted when the incident policy requires it.
- Approved customer notices preserve important facts without diagnosis, blame-shifting, or unreviewed promises.
- The audit trail records creation, classification, manager review, customer communication approval, restriction application/clearance, and closure.

### Failure and exception states

- `IncidentReportIncomplete`: missing pet/reservation/location/time/category/reporter/narrative; create staff follow-up and keep incident `NeedsStaffCompletion`.
- `SeverityUnclassified`: immediate staff note exists but severity is unknown; create lead/manager review before customer message or eligibility clearance.
- `SuspendingIncidentUnreviewed`: group play remains blocked or review-required until manager disposition clears it.
- `CustomerNoticeRequiredButUnapproved`: owner message stays draft-only; no autonomous send.
- `MedicalOrCareAmbiguity`: route to care/document review; do not diagnose or alter medical instructions autonomously.
- `ConflictingSourceFacts`: preserve both staff and customer/source-system facts, mark for manager review, and avoid overwriting the original report.
- `StorageOrSourceSystemWriteFailed`: keep domain decision/audit event pending retry; do not report completion to staff/customer.
- `IncidentClosureDenied`: unresolved restriction/task/review gate prevents closure.

## 2. Domain types to add or refine

Recommended public surface under `operations::daycare`:

```rust
operations::daycare::Incident
operations::daycare::IncidentId
operations::daycare::IncidentReport
operations::daycare::IncidentReportBuilder
operations::daycare::IncidentSource
operations::daycare::IncidentCategory
operations::daycare::IncidentSeverity
operations::daycare::IncidentStatus
operations::daycare::IncidentDisposition
operations::daycare::IncidentRestriction
operations::daycare::IncidentRestrictionStatus
operations::daycare::IncidentReviewRequirement
operations::daycare::IncidentAuditEvent
operations::daycare::IncidentNarrative
operations::daycare::IncidentFactSummary
operations::daycare::IncidentPolicy
operations::daycare::IncidentSeverityPolicy
operations::daycare::IncidentDispositionPolicy
operations::daycare::IncidentEligibilityPolicy
operations::daycare::IncidentCommunicationPolicy
operations::daycare::IncidentRepository
operations::daycare::IncidentTaskRepository
operations::daycare::IncidentWorkflowService
operations::daycare::incident::Result<T>
operations::daycare::incident::Error
```

Refine existing `operations::daycare::IncidentPolicy` from a coarse enum into either a policy config or a small enum plus richer policy services. The existing variants are useful defaults, but later code should not encode all incident behavior in three variants:

```rust
pub enum IncidentPolicy {
    StaffNoteOnly,
    ManagerReviewAndCustomerNotice,
    SuspendGroupPlayPendingReview,
}
```

should become or be wrapped by:

```rust
pub struct IncidentPolicy {
    pub severity_policy: IncidentSeverityPolicyRef,
    pub disposition_policy: IncidentDispositionPolicyRef,
    pub eligibility_policy: IncidentEligibilityPolicyRef,
    pub communication_policy: IncidentCommunicationPolicyRef,
}
```

or a compatibility enum re-exported from `operations::daycare` with explicit policy services as truthful behavior owners.

### Newtypes and invariant-bearing values

- `IncidentId`: provider-neutral non-empty ID or UUID wrapper. Source-system IDs convert explicitly at adapter boundaries.
- `IncidentNarrative`: non-empty, trimmed, length-limited, redacted debug; may contain sensitive staff/customer text.
- `IncidentFactSummary`: non-empty, staff-review-safe summary; customer-safe wording should be a separate drafted message value.
- `IncidentObservedAt`: timestamp value or alias that cannot be absent on submitted incidents.
- `IncidentReviewedAt`, `IncidentClosedAt`: optional values only available through status/disposition transitions, not arbitrary booleans.
- `IncidentMediaRef` / `IncidentDocumentRef`: non-empty references to photos/documents in approved storage; no raw image/document payloads in the domain aggregate.
- `RestrictionDuration`: same-day, until manager review, until temperament reassessment, until medical review, date-bounded, or permanent-until-policy-change; no naked integer days.
- `CustomerNoticeDraftId`: reference to workflow/draft messaging output; not proof of sent notice.

### Enums and invariants

```rust
pub enum IncidentCategory {
    Behavior,
    InjuryOrHealth,
    CareInstructionDeviation,
    FacilityOrSupervision,
    EscapeOrNearMiss,
    CustomerReportedAfterPickup,
    OtherReviewed,
}

pub enum IncidentSeverity {
    NoteOnly,
    OwnerNoticeRequired,
    ManagerReviewRequired,
    SuspendGroupPlayPendingReview,
    EmergencyOrVetEscalation,
}

pub enum IncidentStatus {
    Draft,
    Submitted,
    NeedsStaffCompletion,
    NeedsManagerReview,
    CustomerNoticeDrafted,
    RestrictionActive,
    ResolvedPendingClosure,
    Closed,
    VoidedWithAudit,
}

pub enum IncidentDisposition {
    DocumentOnly,
    OwnerNoticeRequired,
    MoveToIndividualCareForDay,
    SuspendGroupPlayPendingManagerReview,
    RequireTemperamentReassessment,
    RequireCareOrMedicalReview,
    EmergencyOrVetEscalation,
    ClearedForGroupPlayByManager,
}

pub enum IncidentRestriction {
    GroupPlaySuspended { duration: RestrictionDuration },
    IndividualCareOnly { duration: RestrictionDuration },
    ManagerApprovalRequiredBeforeCheckIn,
    CareReviewRequiredBeforeAttendance,
    TemperamentReassessmentRequired,
}
```

Invariants:

- `IncidentStatus::Closed` is impossible while active restrictions, unresolved required review gates, or open incident follow-up tasks remain.
- `IncidentSeverity::SuspendGroupPlayPendingReview` always creates or preserves `IncidentRestriction::GroupPlaySuspended` until a manager-approved disposition clears it.
- `EmergencyOrVetEscalation` cannot be downgraded or customer-closed by an agent; it requires manager/safety handling and audit evidence.
- `DocumentOnly` cannot be used for bite/aggression hard stops, medical/care ambiguity, escape/near-miss, or any policy-classified owner-notice event.
- `ClearedForGroupPlayByManager` requires an existing suspending/review incident, manager identity, rationale, and audit event; agents may draft the packet but cannot create this disposition.
- Customer message drafts and customer notice send proofs are separate values; drafting does not imply approval or delivery.

## 3. Relationship map between types

### Entities and aggregates

- `operations::daycare::Incident` is the aggregate root for daycare incident tracking.
- `operations::daycare::IncidentReport` is the submitted report value used to create the aggregate.
- `entities::PetId`, `entities::CustomerId`, `entities::ReservationId`, `entities::LocationId`, and `entities::StaffId` remain identity references; daycare should not own those identities.
- `operations::daycare::PlaygroupAssignment`, `PlaygroupId`, `CareMode`, and `StaffCoverageDecision` provide incident context when the event involved group play, room/rest lane, or ratio/supervision.
- `temperament::BehaviorObservation`, `temperament::StaffNote`, and `temperament::GroupPlayObservation` provide behavior evidence; final restrictions/dispositions belong to daycare incident policy.
- `care::*` provides medical, allergy, feeding, medication, and review facts; incident workflow routes ambiguity to care/document review.

### Value objects

- `IncidentNarrative`, `IncidentFactSummary`, `IncidentMediaRef`, `RestrictionDuration`, `IncidentObservedAt`, and `CustomerNoticeDraftId` keep raw text, references, and time semantics explicit.
- `IncidentSeverity`, `IncidentDisposition`, `IncidentRestriction`, and `IncidentStatus` are the invariant-bearing center of the model.
- `IncidentAuditEvent` records typed state changes and review actions rather than free-text history rows.

### Policies

- `IncidentSeverityPolicy` classifies report/evidence into severity and review requirements.
- `IncidentDispositionPolicy` decides which dispositions are available from a given incident state, severity, and reviewer authority.
- `IncidentEligibilityPolicy` converts active incident restrictions into `GroupPlayEligibilityDecision::TemporarilySuspended` or `NeedsStaffReview` evidence.
- `IncidentCommunicationPolicy` decides whether a customer notice is required, what review gates apply, and whether an agent may draft but not send.
- `GroupPlayEligibilityPolicy` consumes incident restrictions/evidence but should not own incident classification or closure.

### Repositories/stores

- `operations::daycare::IncidentRepository`: append/read incident aggregates, status transitions, restrictions, audit events, and unresolved incident queries by pet/reservation/location/day.
- `operations::daycare::IncidentTaskRepository`: creates/links `operations::StaffTask` values for follow-up and reads open task state during closure checks.
- `operations::daycare::EligibilityRepository`: invalidates/recomputes daycare group-play eligibility snapshots when incident restrictions change.
- `operations::daycare::RosterRepository`: provides assignment/roster/staff snapshots for incident context; it does not classify incident severity.
- Boundary storage may store provider codes/raw notes, but adapters must promote into semantic incident types before policy decisions.

### Workflow events

- `DaycareIncidentReported`
- `DaycareIncidentSeverityClassified`
- `DaycareIncidentManagerReviewRequested`
- `DaycareIncidentRestrictionApplied`
- `DaycareIncidentCustomerNoticeDrafted`
- `DaycareIncidentCustomerNoticeApproved`
- `DaycareIncidentRestrictionCleared`
- `DaycareIncidentClosed`
- `DaycareIncidentVoidedWithAudit`

These can initially be represented as workflow event names/payloads, but later Rust work should avoid untyped string routing once the workflow event surface is expanded.

### Staff tasks

- `StaffTaskKind::IncidentFollowUp { pet_id }`: default task for incomplete reports, owner notice, manager review, and closure readiness.
- `StaffTaskKind::PlaygroupAssessment { pet_id }`: created when a behavior incident requires reassessment before group play.
- `StaffTaskKind::DocumentReview { pet_id }`: created when vaccine/medical/care source facts need confirmation.
- `StaffTaskKind::DailyUpdateDraft { reservation_id }`: may be created for approved same-day customer updates, but incident/health/safety language still needs review.
- Future refinement: add incident-aware task payloads such as `StaffTaskKind::IncidentFollowUp { incident_id, pet_id }` so task source paths preserve incident identity.

### Agent specs/tools

- `agents::incident-escalation`: summarizes reports, identifies missing fields, drafts manager/owner review packets, proposes severity/disposition, creates internal tasks.
- `agents::daily-care-update`: can draft non-sensitive daily update language from approved staff notes; it must not hide or soften incident facts requiring owner notice.
- `agents::manager-daily-brief`: surfaces unresolved incidents, active restrictions, missed follow-ups, and safety/care risks.
- Tools likely used: `incident-read`, `incident-write-draft` or task-bound incident append, `task-create`, `draft-message`, `care-note-read`, `media-snapshot-read`, `policy-read`. No agent may call a tool that sends incident messages or clears restrictions without review approval.

## 4. Interaction contract

Rust-like pseudo-signatures for later implementation:

```rust
impl operations::daycare::IncidentReport {
    pub fn builder() -> IncidentReportBuilder<MissingPet, MissingLocation, MissingObservedAt, MissingCategory>;
}

impl operations::daycare::Incident {
    pub fn submit(report: IncidentReport, policy: &IncidentPolicy) -> incident::Result<Self>;
    pub fn classify(
        &mut self,
        policy: &impl IncidentSeverityPolicy,
        evidence: IncidentEvidenceSnapshot,
    ) -> incident::Result<IncidentSeverityDecision>;
    pub fn apply_disposition(
        &mut self,
        disposition: IncidentDisposition,
        reviewer: IncidentReviewer,
        policy: &impl IncidentDispositionPolicy,
    ) -> incident::Result<Vec<IncidentAuditEvent>>;
    pub fn close(
        &mut self,
        closure: IncidentClosureRequest,
        tasks: &impl IncidentTaskReadModel,
        policy: &impl IncidentClosurePolicy,
    ) -> incident::Result<IncidentClosed>;
}

pub trait IncidentSeverityPolicy {
    fn classify(
        &self,
        report: &IncidentReport,
        evidence: &IncidentEvidenceSnapshot,
    ) -> IncidentSeverityDecision;
}

pub trait IncidentDispositionPolicy {
    fn allowed_dispositions(
        &self,
        incident: &Incident,
        actor: IncidentActor,
    ) -> Vec<IncidentDispositionOption>;

    fn validate(
        &self,
        incident: &Incident,
        disposition: &IncidentDisposition,
        reviewer: IncidentReviewer,
    ) -> incident::Result<IncidentDispositionApproval>;
}

pub trait IncidentEligibilityPolicy {
    fn eligibility_effect(
        &self,
        pet_id: entities::PetId,
        restrictions: &[IncidentRestriction],
        current: GroupPlayEligibilityDecision,
    ) -> GroupPlayEligibilityDecision;
}

pub trait IncidentCommunicationPolicy {
    fn customer_notice_requirement(
        &self,
        incident: &Incident,
        customer: entities::CustomerId,
    ) -> CustomerNoticeRequirement;
}

pub trait IncidentRepository {
    fn append_report(&self, report: IncidentReport) -> incident::Result<IncidentId>;
    fn load(&self, id: IncidentId) -> incident::Result<Incident>;
    fn append_audit_event(&self, id: IncidentId, event: IncidentAuditEvent) -> incident::Result<()>;
    fn active_restrictions_for_pet(
        &self,
        pet_id: entities::PetId,
        as_of: chrono::DateTime<chrono::Utc>,
    ) -> incident::Result<Vec<IncidentRestriction>>;
    fn unresolved_for_operating_day(
        &self,
        location_id: entities::LocationId,
        day: operations::ResortOperatingDay,
    ) -> incident::Result<Vec<Incident>>;
}

pub trait IncidentTaskRepository {
    fn create_follow_up(&self, task: operations::StaffTask) -> incident::Result<operations::StaffTask>;
    fn open_tasks_for_incident(&self, id: IncidentId) -> incident::Result<Vec<operations::StaffTask>>;
}

pub struct IncidentWorkflowService<R, T, E, C> {
    incidents: R,
    tasks: T,
    eligibility: E,
    communication: C,
}

impl<R, T, E, C> IncidentWorkflowService<R, T, E, C>
where
    R: IncidentRepository,
    T: IncidentTaskRepository,
    E: EligibilityRepository,
    C: CustomerNoticeDraftRepository,
{
    pub fn report_and_route(
        &self,
        report: IncidentReport,
        context: IncidentEvidenceSnapshot,
    ) -> incident::Result<IncidentRoutingOutcome>;

    pub fn apply_manager_disposition(
        &self,
        incident_id: IncidentId,
        disposition: IncidentDisposition,
        manager: entities::StaffId,
    ) -> incident::Result<IncidentDispositionOutcome>;
}
```

Behavior ownership notes:

- `Incident::submit`, `Incident::classify`, `Incident::apply_disposition`, and `Incident::close` own aggregate state transitions and invariant checks.
- Policy objects own classification, allowed disposition, communication, and eligibility-effect decisions.
- Repositories own persistence and query semantics. They should not hide policy decisions in storage helpers.
- `IncidentWorkflowService` coordinates repositories, eligibility invalidation, staff tasks, and customer draft creation. It should not become a generic helper bag; every method should represent a real daycare incident workflow.
- `GroupPlayEligibilityPolicy` consumes active restrictions but does not close incidents or clear suspensions.

## 5. Review and approval contract

### Automation level

- Safe to automate: read incident/report/policy context; detect missing required fields; summarize facts; classify a proposed severity; create internal staff/manager review tasks; draft manager packets; draft customer message language with explicit review gates; add incident facts to manager daily brief.
- Internal-task-only: creating `IncidentFollowUp`, `PlaygroupAssessment`, `DocumentReview`, and manager-review tasks.
- Draft-only: customer/owner notice text, pickup conversation summaries, and daily-care updates that mention incidents, health, or behavior concerns.
- Manager approval required: severity downgrade, group-play suspension, group-play reinstatement, emergency/vet/safety escalation closure, customer-facing incident message approval, refund/credit/waiver suggestions, and any local policy override.
- Never automate: diagnose medical/behavioral conditions, send owner-facing incident or health messages, hide concerning facts, clear suspending restrictions, close severe incidents, alter staff schedules, or mutate source-system records without approved tool contract.

### Review gates

- `policy::ReviewGate::ManagerApproval`: required for severe incidents, suspensions, reinstatements, emergency/vet escalations, conflicting facts, and closure denial overrides.
- `policy::ReviewGate::BehaviorReview`: required for behavior incidents that affect group-play eligibility or reassessment.
- `policy::ReviewGate::MedicalDocumentReview`: required for health/care/medical/vaccine ambiguity.
- `policy::ReviewGate::CustomerMessageApproval`: required for any customer-facing incident/health/safety/behavior message.
- `policy::ReviewGate::RefundOrDepositException`: required if incident handling suggests credits, refunds, discounts, or waived fees.

### Audit trail

Every state transition should append an `IncidentAuditEvent` with actor, timestamp, source, prior state, new state, rationale, and review evidence reference. Required audited actions:

- report submitted or amended,
- severity classified or reclassified,
- staff task created/completed,
- review gate requested/satisfied/denied,
- customer notice drafted/approved/sent proof recorded,
- eligibility restriction applied/cleared,
- manager disposition applied,
- incident closed or voided.

### Customer/member-facing boundaries

- Customer messages are drafts until explicitly approved; draft IDs are not delivery proof.
- Messages should be factual, non-diagnostic, and should not omit material incident facts merely to protect sentiment.
- Agents may suggest wording but must preserve unresolved facts and review requirements in the packet.
- No agent can promise refund/credit/vet reimbursement/behavior clearance or future availability.
- Staff and manager notes may contain sensitive internal detail; customer-safe summaries must be separate values with approval state.

## 6. Test contracts

Named semantic tests for later code cards:

1. `incident_report_requires_pet_location_time_category_reporter_and_narrative`
   - `IncidentReport::builder()` cannot submit without core report fields; incomplete reports route to `NeedsStaffCompletion`.

2. `note_only_incident_does_not_change_group_play_eligibility`
   - A policy-classified note-only event records audit facts but does not create active restrictions.

3. `bite_or_aggression_incident_suspends_group_play_pending_manager_review`
   - Behavior hard-stop evidence creates `IncidentRestriction::GroupPlaySuspended` and invalidates current `GroupPlayEligibilityDecision` to `TemporarilySuspended` or review-required.

4. `medical_or_care_ambiguity_routes_to_care_review_not_diagnosis`
   - Health/care incident evidence creates medical/care review gates and forbids automated diagnosis or autonomous handling-instruction changes.

5. `owner_notice_required_incident_creates_customer_message_draft_with_review_gate`
   - Owner-notice severity produces a draft and `ReviewGate::CustomerMessageApproval`; no send-proof is recorded until approval/delivery occurs.

6. `manager_review_required_incident_cannot_be_closed_by_agent_disposition`
   - Agent-proposed disposition can create a packet/task but cannot close or clear the incident.

7. `suspending_incident_blocks_group_assignment_until_restriction_cleared`
   - `AssignmentService` refuses `CareMode::DogGroupPlay` while an active group-play suspension exists, even if other eligibility evidence is favorable.

8. `manager_clearance_requires_reviewer_identity_rationale_and_audit_event`
   - Clearing a suspension requires manager actor, rationale, and audit append; missing evidence returns a semantic incident error.

9. `incident_closure_fails_with_open_follow_up_tasks_or_active_restrictions`
   - `Incident::close` returns `IncidentClosureDenied` while tasks/restrictions/review gates remain unresolved.

10. `customer_reported_after_pickup_preserves_conflicting_source_facts`
    - Customer-reported facts do not overwrite original staff report; conflicting facts route to manager review with both sources preserved.

11. `emergency_or_vet_escalation_requires_manager_or_safety_review_before_closure`
    - Severe escalation cannot be downgraded or closed without manager/safety review evidence.

12. `incident_daily_brief_surfaces_unresolved_restrictions_and_safety_risks`
    - Manager daily brief includes unresolved incident restrictions, overdue follow-up tasks, and `PetSafetyOrCareRisk` entries.

13. `incident_storage_roundtrip_preserves_semantic_status_severity_and_restrictions`
    - Storage adapters reject unknown required enum values and do not silently default incident severity/status/restriction fields.

14. `customer_safe_summary_is_distinct_from_internal_staff_narrative`
    - Internal narratives and customer-safe summaries are separate values with separate redaction/debug behavior and approval state.

15. `daycare_incident_paths_preserve_daycare_meaning`
    - Public examples use `operations::daycare::Incident`, `operations::daycare::IncidentDisposition`, and `operations::daycare::IncidentRepository`, not generic `incident::process()` helpers or raw strings.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Extend or split `operations::daycare` incident types.
  - Consider moving the large daycare surface into `domain/src/operations/daycare.rs` if the module grows beyond the current single-file pattern.
  - Refine `IncidentPolicy` while preserving or intentionally migrating existing tests.
- `domain/src/policy.rs`
  - Reuse `ReviewGate`, `AutomationLevel`, and policy denial semantics; add incident-specific denial/review reasons only if they are not daycare-owned values.
- `domain/src/agents.rs`
  - Refine `incident-escalation` spec tools/forbidden actions if incident repositories/tools become explicit.
- `domain/src/tools.rs`
  - Add or type incident read/write-draft/task tool boundaries if tool contracts are modeled in this crate.
- `domain/src/workflow.rs`
  - Add typed incident workflow events or payload names after the aggregate contract is stable.
- `domain/src/temperament.rs`
  - Ensure behavior observations can feed incident severity/eligibility policies without becoming final disposition values.
- `domain/src/care.rs`
  - Link medical/care review requirements to incident routing without importing daycare into care core unless a true shared concept emerges.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Extend daycare contract tests for incident policy/refined incident values.
- `domain/tests/domain_quality_patterns.rs`
  - Add semantic path/no-helper/no-raw-string tests for incident tracking if this project keeps doctrine tests there.

### Migration/refactor risks

- Existing `operations::daycare::IncidentPolicy` is currently a simple enum. Replacing it directly may break `Contract::standard_petsuites()` and existing contract tests. Prefer a staged migration: add new incident types and policy services, keep compatibility enum/re-export, then migrate call sites.
- `operations::StaffTaskKind::IncidentFollowUp { pet_id }` lacks `incident_id`. Later incident workflows will need linked tasks; adding a new field is a breaking enum change. Consider a new variant or source link through `StaffTaskSource::WorkflowEvent` first.
- `policy::PlayEligibilityDecision` is currently conservative and pet/service-based. Incident restrictions should feed a daycare-owned eligibility layer rather than bloating the generic policy trait with daycare incident persistence.
- Medical/health incidents may tempt the model to encode diagnosis. Keep care/medical review values factual and route decisions to human review.
- Source systems may expose incident status/severity as raw strings or notes. Storage adapters must reject unknown required values or quarantine them as review-required; they must not default to `NoteOnly`.
- Customer message workflows may conflate draft, approval, and delivery. Model these as distinct values to preserve audit truth.

### Dependencies on other implications

- Group-play eligibility: incident restrictions are input evidence for eligibility decisions and must be able to invalidate or suspend prior eligibility snapshots.
- Playgroup assignment/staff coverage: incident reports may need the roster, group, room, ratio, and staff coverage snapshot at observed time.
- Daily care updates: approved incident facts can inform same-day customer updates, but sensitive incident/health/behavior wording requires customer-message approval.
- Package/membership opportunities: severe or unresolved incidents should suppress or mark revenue follow-up suggestions for staff review; agents must not recommend upsell language that ignores active safety restrictions.
- Front-desk readiness/check-in: active incident restrictions or unresolved reviews should make a future visit not-ready until review clears.

Implementation order recommendation:

1. Add incident newtypes/enums, semantic errors, and report builder tests.
2. Add aggregate transition methods for submit, classify, apply disposition, apply/clear restriction, and close.
3. Add policy traits and deterministic tests for severity, disposition, eligibility effect, communication gates, and closure denial.
4. Add repository traits/read models only after aggregate and policy contracts are green.
5. Wire staff task/workflow/customer-draft contracts behind review gates.
6. Add storage roundtrip tests for semantic incident values and adapters that quarantine unknown source-system values.

Doc-only status: this artifact maps the intended incident-tracking domain shape. It performs no live, destructive, member-facing, storage, source-system, or policy-changing action.
