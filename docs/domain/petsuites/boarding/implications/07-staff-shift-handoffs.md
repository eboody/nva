# Boarding implication 07: Staff shift handoffs

Purpose: define the Boarding staff-shift handoff contract for later Rust implementation. This is a modeling artifact only; it does not authorize live task updates, reservation changes, customer messages, payment actions, or member-facing automation.

Assumptions:

- A shift handoff is an internal operational artifact produced at a location boundary between outgoing and incoming staff. It summarizes obligations that still matter for a boarding stay after the outgoing shift ends.
- The safest default is conservative: missing, ambiguous, or sensitive evidence becomes a typed review/escalation item, not an agent-written conclusion.
- The first code card should preserve the existing `operations::boarding::HandoffRequirement` enum while introducing richer `operations::boarding::handoff::*` types around it; do not replace the existing service-contract field with a parallel concept until storage migration is explicit.

## 1. Operational story

### Trigger

A handoff packet is prepared whenever one of these internal events occurs:

- Scheduled shift boundary for kennel, front desk, lead staff, or manager coverage.
- Boarding arrival/check-in is incomplete before the outgoing shift ends.
- Boarding checkout/departure prep is incomplete before the outgoing shift ends.
- A care obligation, staff task, manager review, capacity/room issue, incident, medication, feeding exception, or customer follow-up remains open.
- An agent or daily-brief workflow detects unresolved Boarding work and drafts a packet for staff review.

The trigger should be modeled as `operations::boarding::handoff::Trigger`, not as a raw string or generic cron label.

### Actors

- Outgoing staff: kennel technicians, front desk, groomers/trainers attached to add-ons, lead staff, or manager. They can attest to task state and add evidence.
- Incoming staff: next shift role/staff member receiving obligations and review gates.
- Shift lead: validates packet completeness and resolves ordinary blocked tasks.
- Manager: approves safety-critical, incident, capacity, medication ambiguity, deposit/payment, cancellation, complaint, or policy-exception escalations.
- Agent/workflow: read-only summarizer/drafter that can propose packet contents and internal staff tasks; it cannot attest completion, suppress risks, or message customers.

### Inputs

- `entities::LocationId` and operating-day/shift window.
- `entities::ReservationId`, `CustomerId`, `PetId`, species, accommodation assignment, and stay date range projections.
- Boarding contract policy: `operations::boarding::Contract`, current `HandoffRequirement`, housekeeping cadence, service windows, deposit/payment timing, and upsell/report policy.
- Open `operations::StaffTask` values and Boarding-owned `operations::boarding::task::Kind` projections.
- Care evidence from `entities::CareProfile`, `TemperamentProfile`, `care::*`, medication/feeding reviews, task completion evidence, Pawgress/report evidence, incident flags, and staff notes.
- Capacity/room evidence: accommodation assignment, room turnover state, capacity holds, room availability, late check-out risk, and waitlist/manager-hold decisions.
- Workflow/audit context: prior `workflow::WorkflowEvent`s, `policy::ReviewGate`s, `RecommendedAction`s, and current approval state.

### Decisions

The handoff policy answers:

- Which open obligations must be carried into the next shift?
- Which obligations are informational, shift-lead required, manager required, or safety-critical?
- Which task completion evidence is strong enough to mark an item complete?
- Which care/report/customer-facing facts are too sensitive or ambiguous for agent summary without review?
- Whether a packet is ready for shift-lead sign-off, requires manager review, or must remain blocked.
- Whether any customer-facing draft/report/payment/reservation action is merely attached as pending context rather than executed.

### Outputs

- `operations::boarding::handoff::Packet` with typed shift window, source trigger, location, reservations/pets, open obligations, blocked care items, review gates, manager escalations, and evidence references.
- `operations::boarding::handoff::Item` entries carrying semantic owner, subject, severity, due timing, current evidence, and acceptance criteria.
- Internal `operations::StaffTask` drafts for the incoming role when an obligation needs assignment.
- `workflow::RecommendedAction::InternalTask` or manager-escalation recommendations where policy says review is required.
- `entities::AuditEvent` / `workflow::WorkflowEvent` entries that record who drafted, reviewed, amended, accepted, or escalated the packet.

### Success state

A handoff succeeds when:

- Every continuing Boarding obligation has exactly one explicit next owner: incoming role/staff, shift lead, manager, or intentionally deferred pending external/customer evidence.
- Safety-critical, medication, behavior, incident, payment/deposit, cancellation, and customer-facing items carry typed review gates.
- Completed items have completion evidence strong enough for their kind; ambiguous text/media remains review-required.
- Incoming staff can execute the next shift without reconstructing context from raw reservation notes.
- The packet is auditable: trigger, input snapshot IDs, policy version, actor, review decision, and amendments are retained.

### Failure and exception states

- `NoIncomingOwner`: an open obligation cannot be assigned to a role/staff member.
- `MissingCareEvidence`: feeding, medication, behavior, accommodation, belongings, or report evidence is absent or contradictory.
- `AmbiguousCompletionEvidence`: free-text/media evidence cannot safely close the obligation.
- `ManagerReviewRequired`: policy exception, incident, sensitive customer-facing content, capacity hold, deposit/payment exception, or medication ambiguity requires manager approval.
- `SafetyCriticalEscalation`: animal safety, injury, bite/aggression, medical distress, escaped/lost pet, or facility hazard must be escalated immediately and cannot wait for ordinary shift acceptance.
- `PacketStale`: packet source snapshot is older than the configured freshness window or superseded by later staff action.
- `CrossServiceDependencyBlocked`: grooming/training/exit-bath/add-on task is still pending and owned by another service line.

## 2. Domain types to add or refine

### Keep and refine existing surface

- `operations::boarding::HandoffRequirement`
  - Current variants: `ArrivalCareReview`, `MedicationDoubleCheck`, `DepartureBelongingsReview`.
  - Refine as the service-contract default requirement: what a PetSuites location requires by default, not the full runtime packet.
  - Add only if necessary: `ShiftLeadAcceptance`, `ManagerEscalationForSensitiveItems`, `SafetyCriticalImmediateEscalation`.

### New `operations::boarding::handoff` module

- `handoff::Packet`
  - Aggregate for one shift transition at one location.
  - Invariants: non-empty `items`; one `location_id`; packet window has `outgoing_shift.ends_at <= incoming_shift.starts_at` or a documented overlap; each item subject belongs to the packet location/snapshot; every unresolved item has a next owner or review gate.

- `handoff::Id`
  - Opaque durable packet identity. Non-empty if provider/external; UUID/newtype if internal.

- `handoff::Trigger`
  - `ScheduledShiftChange`, `ArrivalCareReviewIncomplete`, `MedicationDoubleCheckDue`, `DepartureBelongingsReviewDue`, `HousekeepingOrTurnoverIncomplete`, `IncidentOrSafetyFollowUp`, `ManagerReviewCarryover`, `AgentDraftFromDailyBrief`.

- `handoff::ShiftWindow`
  - Start/end time and role context for outgoing/incoming shift.
  - Invariant: end follows start; location timezone is explicit or inherited from `entities::Location` snapshot.

- `handoff::Item`
  - A single continuing obligation.
  - Fields: `subject`, `kind`, `severity`, `status`, `next_owner`, `due`, `evidence`, `review_gate`, `source_task`, `acceptance_criteria`.
  - Invariant: `SafetyCritical` items must carry manager/lead escalation and cannot be `InfoOnly`; completed items must carry typed evidence.

- `handoff::ItemKind`
  - `ArrivalCareReview`, `MedicationDoubleCheck`, `FeedingException`, `PottyWalkDue`, `HousekeepingDue`, `RoomTurnover`, `PlaytimeEligibilityReview`, `PawgressReportDraftReview`, `CheckoutBelongingsReview`, `DepositOrPaymentFollowUp`, `CapacityOrRoomHold`, `IncidentFollowUp`, `CustomerMessageDraftPendingReview`, `CrossServiceAddOnPending`.

- `handoff::Subject`
  - `Reservation(entities::ReservationId)`, `Pet { reservation_id, pet_id }`, `Room { reservation_id, room_id }`, `Customer { reservation_id, customer_id }`, `Location(entities::LocationId)`.
  - This prevents vague handoff notes that cannot be traced to a pet/reservation/location.

- `handoff::Severity`
  - `Info`, `NeedsIncomingStaff`, `NeedsShiftLead`, `NeedsManager`, `SafetyCritical`.

- `handoff::Status`
  - `Draft`, `ReadyForShiftLeadReview`, `AcceptedByIncomingShift`, `Blocked`, `Escalated`, `Superseded`, `Closed`.

- `handoff::Owner`
  - `Role(operations::StaffRole)`, `Staff(entities::StaffId)`, `Manager(entities::ManagerId)`, `PendingExternalEvidence`, `CrossService { service: entities::ServiceKind }`.

- `handoff::Evidence`
  - `TaskCompletion(operations::TaskCompletionEvidence)`, `StaffAttestation { actor, note }`, `CareProfileSnapshot`, `MedicationReview`, `FeedingReview`, `RoomInspection`, `MediaReference`, `WorkflowEvent(workflow::WorkflowEventId)`, `NoneYet`.
  - Sensitive content should be referenced/redacted, not copied into broad debug/audit output.

- `handoff::AcceptanceDecision`
  - `Accepted`, `AcceptedWithManagerEscalation`, `RejectedAsIncomplete { reasons }`, `SupersededByNewerPacket { replacement }`.

- `handoff::Policy`
  - Deterministic owner of handoff inclusion, severity, evidence sufficiency, and review-gate rules.

- `handoff::Repository`
  - Stores/retrieves packet snapshots and decisions. It should store semantic records, not raw provider note blobs.

- `handoff::Planner`
  - Domain service that builds packet drafts from reservations, tasks, care evidence, and policy snapshots.

## 3. Relationship map

### Entities

- `entities::LocationId`: packet scope and policy selection.
- `entities::ReservationId`: anchors stay-level handoff items.
- `entities::PetId`: anchors pet-specific care, medication, feeding, playtime, incident, or report items.
- `entities::CustomerId`: anchors customer follow-up/report/payment context; does not authorize messaging.
- `entities::StaffId`, `ManagerId`, `ActorRef`: actor identity for attestations, reviews, and audit events.

### Value objects

- `handoff::ShiftWindow`, `handoff::Trigger`, `handoff::Severity`, `handoff::Status`, `handoff::Owner`, `handoff::Evidence`, `handoff::AcceptanceDecision`.
- Existing `operations::boarding::HandoffRequirement`, `HousekeepingCadence`, `ServiceWindow`, `PaymentTiming` remain policy inputs.
- Potential room/accommodation types from the core map: `accommodation::RoomId`, `accommodation::Assignment`, `capacity::Snapshot`.

### Policies

- `boarding::handoff::Policy`: item inclusion, owner/severity/review gate, packet freshness, evidence sufficiency.
- `boarding::care::Policy`: feeds medication/feeding/behavior review outcomes and unresolved care gates.
- `boarding::report::Policy`: Pawgress/customer-facing content approval state.
- `boarding::agent::ApprovalPolicy`: maps agent-drafted packet outputs to internal-only or review-required automation levels.
- `policy::ReviewGate` and `policy::AutomationLevel`: general approval vocabulary.

### Repositories/stores

- `boarding::handoff::Repository`: packet snapshots, decisions, amendments.
- `boarding::reservation::Repository`: date/location/stay projections and reservation status.
- `boarding::care::Repository`: care profile projections and task/evidence reads.
- `boarding::capacity::Repository` / `room::Repository`: room assignments, turnover state, availability/hold evidence.
- `operations::staff_task::Repository` or existing operations task store: staff task drafts and completion evidence.
- `storage::operations`: should serialize future packet records through semantic conversions.

### Workflow events

- `handoff::Drafted`, `ItemAdded`, `ItemEscalated`, `ShiftLeadReviewed`, `IncomingShiftAccepted`, `ManagerApproved`, `PacketSuperseded`, `PacketClosed` should be represented as typed workflow/audit events or structured metadata under `workflow::WorkflowEvent` / `entities::AuditEvent`.

### Staff tasks

- Boarding-owned task kinds map to existing `operations::StaffTaskKind` without losing source semantics:
  - `handoff::ItemKind::MedicationDoubleCheck` -> `StaffTaskKind::MedicationAdministration { pet_id }` plus review gate/source.
  - `FeedingException` -> `StaffTaskKind::Feeding { pet_id }`.
  - `RoomTurnover`/`HousekeepingDue` -> `StaffTaskKind::CleaningTurnover { reservation_id }`.
  - `CheckoutBelongingsReview` -> `StaffTaskKind::CheckOutPrep { reservation_id }`.
  - `PawgressReportDraftReview` -> `StaffTaskKind::DailyUpdateDraft { reservation_id }`.
  - `IncidentFollowUp` -> `StaffTaskKind::IncidentFollowUp { pet_id }`.

### Agent specs/tools

- Agent spec: `agents::WorkflowAgent` / `agent::Spec` named `boarding-shift-handoff-drafter`.
- Allowed tools: read reservations, read staff tasks, read care/temperament summary, read capacity/room projection, draft internal task, draft handoff packet.
- Forbidden tools/actions: confirm/cancel/modify reservation, charge/refund/waive deposits, send customer messages/Pawgress reports, mark care tasks complete from ambiguous evidence, suppress safety/incident facts.

## 4. Interaction contract

Rust-like pseudo-signatures use semantic module paths and behavior owners. Exact lifetimes/error composition can be chosen by the implementation card.

```rust
impl operations::boarding::handoff::Policy {
    pub fn classify_item(
        &self,
        item: &operations::boarding::handoff::ItemDraft,
        context: &operations::boarding::handoff::Context,
    ) -> operations::boarding::handoff::Classification;

    pub fn evidence_suffices(
        &self,
        kind: operations::boarding::handoff::ItemKind,
        evidence: &operations::boarding::handoff::Evidence,
    ) -> operations::boarding::handoff::EvidenceDecision;

    pub fn review_gate_for(
        &self,
        item: &operations::boarding::handoff::Item,
    ) -> Option<policy::ReviewGate>;
}
```

Behavior belongs on `handoff::Policy` because inclusion/severity/evidence sufficiency are policy decisions, not generic helpers.

```rust
impl operations::boarding::handoff::Planner {
    pub fn draft_packet(
        &self,
        trigger: operations::boarding::handoff::Trigger,
        shift: operations::boarding::handoff::ShiftWindow,
        snapshot: operations::boarding::handoff::InputSnapshot,
    ) -> operations::boarding::handoff::Result<operations::boarding::handoff::Packet>;
}
```

`Planner` owns packet assembly. It may consume repository snapshots, but it does not mutate live reservations or complete staff tasks.

```rust
pub trait operations::boarding::handoff::Repository {
    fn save_draft(&self, packet: &handoff::Packet) -> handoff::Result<handoff::Id>;
    fn load(&self, id: handoff::Id) -> handoff::Result<handoff::Packet>;
    fn record_decision(
        &self,
        id: handoff::Id,
        decision: handoff::AcceptanceDecision,
        actor: entities::ActorRef,
    ) -> handoff::Result<()>;
    fn latest_for_shift(
        &self,
        location: entities::LocationId,
        shift: handoff::ShiftWindow,
    ) -> handoff::Result<Option<handoff::Packet>>;
}
```

The repository owns persistence and audit linkage for packets; it should not decide what belongs in a packet.

```rust
impl operations::boarding::handoff::Packet {
    pub fn requires_manager_review(&self) -> bool;
    pub fn unresolved_items(&self) -> impl Iterator<Item = &handoff::Item>;
    pub fn accept(
        self,
        actor: entities::ActorRef,
        decision: handoff::AcceptanceDecision,
    ) -> handoff::Result<handoff::Packet>;
}
```

The packet owns state transitions that depend only on its current state and typed decision. It should reject acceptance if safety-critical or manager-gated items are unresolved.

```rust
impl operations::boarding::task::Mapper {
    pub fn staff_task_for_handoff_item(
        &self,
        item: &operations::boarding::handoff::Item,
    ) -> operations::boarding::task::MappingDecision;
}
```

Mapping belongs to Boarding task semantics, not the generic `operations::StaffTaskKind`, because Boarding knows why the task exists.

```rust
impl operations::boarding::agent::ApprovalPolicy {
    pub fn handoff_output_level(
        &self,
        output: &operations::boarding::handoff::AgentOutput,
    ) -> policy::AutomationLevel;
}
```

Agent output can be internal draft/summarization only. Any downstream live execution still requires typed review/approval.

## 5. Review and approval contract

### Automation level

- Agent may fully automate read-only extraction and draft generation of internal handoff packets from already-available task/reservation/care/capacity evidence.
- Agent may create internal task drafts if `boarding::agent::ApprovalPolicy` classifies them as internal-only and not sensitive live changes.
- Agent may not mark tasks complete, attest medication/feeding correctness, close incidents, assign blame, suppress negative facts, or decide customer-facing report/message readiness.

### Review gates

Staff review required:

- Ordinary incoming shift acceptance.
- Free-text/media task evidence before completion closure.
- Pawgress report/customer update drafts attached to packet.
- Playtime eligibility or behavior review when not deterministically clean.
- Arrival/departure changes based on customer messages.

Manager review required:

- Safety-critical or incident follow-up.
- Medication ambiguity, vet clarification, injury/illness, bite/aggression, escaped/lost pet, facility hazard.
- Capacity/room hold overrides, overbooking, waitlist exception, minimum-stay exception.
- Deposit/payment waiver/refund/forfeit, cancellation exception, complaint/legal/regulatory signal.
- Sensitive customer-facing explanation about medical, behavior, incident, payment, denial, or policy decisions.

### Audit trail

Every packet should retain:

- Trigger, source snapshots, policy version, location, shift window, and generated-at time.
- Agent/staff/manager actor for draft, edit, sign-off, escalation, and closure.
- Per-item source task/evidence/review gate and changes made during review.
- Supersession linkage when a packet is replaced by a newer snapshot.

### Customer/member-facing boundaries

Shift handoff packets are internal. They may reference customer-facing drafts, Pawgress reports, reservation actions, or payment actions as pending obligations, but they do not send or execute them. Tool adapters must reject any handoff-derived action that lacks the explicit approval contract required by the underlying customer/payment/reservation/report domain.

## 6. Test contracts

Future implementation should add semantic tests with names like:

- `boarding_handoff_packet_requires_at_least_one_unresolved_or_context_item`
  - Empty packets are invalid unless a separate `NoHandoffNeeded` decision exists.

- `boarding_handoff_packet_assigns_every_unresolved_item_to_next_owner_or_review_gate`
  - Open items cannot be orphaned across shift boundary.

- `boarding_handoff_policy_escalates_medication_ambiguity_to_manager_or_shift_lead_review`
  - Medication uncertainty does not become an ordinary info note.

- `boarding_handoff_policy_marks_incident_followup_as_safety_critical`
  - Incident/safety item severity is not flattenable to a generic task.

- `boarding_handoff_packet_includes_open_medication_feeding_housekeeping_and_checkout_tasks`
  - Carries forward the acceptance-test contract named in the Boarding service map.

- `boarding_handoff_evidence_decision_rejects_empty_or_ambiguous_completion_text_for_sensitive_items`
  - Free text alone cannot close medication, feeding exception, incident, or behavior-sensitive work.

- `boarding_handoff_packet_cannot_be_accepted_while_manager_gated_item_is_unresolved`
  - Incoming shift acceptance cannot bypass manager review.

- `boarding_handoff_agent_can_draft_internal_packet_but_cannot_mark_staff_task_complete`
  - Approval policy keeps agent output internal/draft-only.

- `boarding_handoff_task_mapping_preserves_boarding_item_kind_and_source_task`
  - Mapping into `operations::StaffTaskKind` keeps Boarding subject/source/reason.

- `boarding_handoff_packet_records_audit_actor_for_draft_review_acceptance_and_supersession`
  - Audit trail preserves typed actors and lifecycle.

- `boarding_handoff_packet_redacts_sensitive_care_evidence_in_debug_output`
  - Debug/log output cannot leak medication, feeding, allergy, incident, or staff-note details.

- `boarding_handoff_packet_stale_snapshot_requires_redraft_before_acceptance`
  - Stale packets cannot be accepted after newer staff/task evidence exists.

- `boarding_handoff_attached_customer_message_remains_draft_until_customer_message_review_gate_clears`
  - Handoff context never sends a customer/member-facing message.

## 7. Integration notes for serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add `operations::boarding::handoff` module or split `operations` into service-line submodules if that refactor is already underway.
  - Preserve existing `operations::boarding::Contract` and `HandoffRequirement` as compatibility surfaces.
  - Add `handoff::Packet`, `Item`, `Policy`, `Planner`, semantic enums/newtypes, and `Result/Error` if implementing behavior.

- `domain/src/workflow.rs`
  - Add event types or structured subjects/actions if current `WorkflowEvent` cannot represent packet draft/review/accept/supersede semantics.

- `domain/src/entities.rs`
  - Confirm `ActorRef`, `AuditEvent`, `AuditSubject`, and metadata contracts can reference handoff packets without raw strings; add typed subject/action variants only if needed.

- `domain/src/agents.rs` and `domain/src/agent.rs`
  - Add/extend agent spec for `boarding-shift-handoff-drafter` and forbidden-action/default-review-gate tests.

- `domain/src/tools.rs`
  - Ensure any handoff-derived tool boundary is draft-only for staff tasks and rejects live customer/payment/reservation actions without approval.

- `domain/tests/petsuites_core_service_contracts.rs`
  - Add contract-level tests that `standard_petsuites()` still encodes a handoff requirement and storage roundtrips preserve it.

- `domain/tests/domain_quality_patterns.rs`
  - Add semantic tests for packet invariants, redaction/debug safety, audit actors, and agent approval boundaries.

- Storage crate/tests for `storage::operations::CoreServiceContractsRecord` or a future `storage::operations::boarding::handoff::PacketRecord`.

### Migration/refactor risks

- Existing `HandoffRequirement` is a single contract enum. Do not overload it with runtime packet state; introduce `handoff::Packet` and bridge from the contract requirement.
- Existing `operations::StaffTaskKind` is generic. Adding many Boarding-only variants there may flatten the domain; prefer Boarding-owned `task::Kind` or handoff item kinds with a mapper.
- Care evidence can contain sensitive staff notes, medication details, allergies, incident descriptions, and customer context. Avoid `#[derive(Debug)]` on structures that print raw sensitive fields unless redaction is explicit.
- Storage serialization currently roundtrips compact service contracts. Rich packet storage needs versioned records and semantic conversions, not provider JSON leaking into the domain core.
- Handoff packet acceptance is a state transition. If runtime enums become insufficient, consider typestate only for phases with materially different legal methods (`Draft` -> `Reviewed` -> `Accepted` -> `Closed`).
- Cross-service add-ons such as grooming/training/exit bath should be referenced as dependencies, not absorbed into Boarding ownership.

### Dependencies on other implications

- Capacity/room assignment implication: handoff items for room holds, turnover, late check-out, and room assignment need typed room/accommodation/capacity snapshots.
- Care/medication/feeding implication: handoff severity and review gates depend on care policy outputs.
- Check-in/check-out implication: arrival care review, checkout belongings, payment, Pawgress/report readiness, and departure tasks are primary handoff sources.
- Pawgress Report/customer communication implication: handoff can carry report draft review obligations but must not send them.
- Agent approval-boundary implication: handoff drafter must share the same deterministic `boarding::agent::ApprovalPolicy` model.

### Recommended first implementation slice

1. Add `operations::boarding::handoff::{Trigger, Severity, Status, Owner, Subject, ItemKind, Item, Packet}` with constructors enforcing non-empty packet items and explicit owner/review gate for unresolved items.
2. Add `handoff::Policy` methods for item severity/review-gate decisions, focused first on medication, feeding, housekeeping, checkout belongings, and incident follow-up.
3. Add task mapping from `handoff::ItemKind` to existing `operations::StaffTaskKind` while preserving Boarding source/reason.
4. Add tests for packet invariants, manager-gated acceptance, agent draft-only boundary, and debug redaction before wiring storage.
