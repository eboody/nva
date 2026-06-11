# Grooming implication 03: no-show/cancellation management

Purpose: define the operational domain contract for grooming no-show and cancellation management. This is a modeling artifact for later serialized Rust/domain cards. It refines the grooming service-domain map without changing code, and keeps provider payloads, payment execution, calendar mutation, and customer messaging behind typed policy and approval boundaries.

Safe assumption: PetSuites locations may vary in notice windows, deposit amounts, waitlist behavior, and whether a late cancellation counts the same as a no-show. The safest extensible model is to preserve distinct semantic outcomes (`CancelledWithNotice`, `LateCancellation`, `NoShow`) while allowing a location contract to map them to deposit, hold-release, rebooking, and review decisions.

## 1. Operational story

### Trigger

The workflow starts when one of these events is observed:

- a customer requests cancellation or reschedule for a scheduled grooming appointment;
- a groomer/front-desk staff member marks the pet/customer absent after the appointment start grace period;
- a provider system emits a cancellation/no-show status;
- an agent or daily brief detects a missed appointment, late cancellation pattern, or rebooking attempt with prior no-show/cancellation history.

### Actors

- Customer/member: requests cancellation/reschedule, receives approved communication, may owe or pay a rebooking deposit.
- Front-desk staff: records attendance/cancellation facts, releases calendar slot, drafts/resolves customer follow-up, verifies provider status.
- Groomer: confirms no-show after grace period, may request manager review when the missed slot affected utilization or pet handling preparation.
- Manager: approves penalty/deposit exceptions, waivers, repeated-history rebooking decisions, and customer-sensitive escalations.
- Grooming rebooking agent: reads history/calendar/policy context, drafts recommendations and messages, and flags review requirements; it never books, cancels, charges, waives, or sends on its own.
- Provider/tool adapters: read/write external calendar, appointment status, deposit records, audit entries, and task/message drafts only after deterministic policy approval.

### Inputs

- `operations::grooming::AppointmentId` or provider appointment reference promoted at the boundary.
- `entities::LocationId`, `entities::CustomerId`, `entities::PetId`, optional `entities::ReservationId` when grooming is tied to boarding/daycare checkout.
- Scheduled start/end, service offering, groomer assignment, deposit status/reference, and appointment state snapshot.
- Cancellation request time, actor, channel, customer reason/category, and requested replacement window if known.
- Attendance/no-show observation time, grace-period policy, confirming staff/groomer actor, and evidence note.
- Location grooming contract: current `operations::grooming::NoShowPolicy`, future `CancellationPolicy`, release/waitlist rules, manager-review thresholds, and automation boundary.
- Customer/pet grooming attendance history: no-show count, late cancellation count, prior waivers, outstanding deposit requirement, and manager decisions.
- Customer communication consent/channel preferences from the customer context.

### Decisions

1. Classify the event semantically: on-time cancellation, late cancellation, no-show, provider status correction, customer disputed event, or duplicate event.
2. Determine calendar effect: keep appointment, release slot, offer waitlist fill, create draft hold for reschedule, or require manual calendar review.
3. Determine rebooking eligibility: rebook freely, require deposit before rebooking, allow staff-reviewed hold, require manager review, or block until outstanding balance/policy issue is resolved.
4. Determine payment/deposit decision: no deposit action, deposit required, deposit forfeiture candidate, waiver candidate, refund/exception candidate, or payment-boundary review.
5. Determine communication boundary: internal note only, staff task, customer-message draft, or manager-approved customer response.
6. Determine audit obligation: record policy decision, actor, evidence, appointment/history references, and any approval token used before external execution.

### Outputs

- `operations::grooming::attendance::Outcome` or equivalent appointment-history event with typed outcome, occurred-at time, actor, and evidence.
- `operations::grooming::NoShowDecision` / `CancellationDecision` explaining rebooking, deposit, calendar-release, task, and communication consequences.
- Draft `operations::StaffTask` for follow-up, manager review, provider correction, or waitlist fill when policy requires human handling.
- Draft customer message packet when communication is useful and consent/review gates allow a draft.
- Payment/deposit command draft or requirement reference; no payment capture, waiver, refund, or forfeiture execution happens inside grooming policy.
- Calendar command draft such as `ReleaseSlot`, `CreateDraftHold`, or `OfferWaitlistFill`; no confirmed provider booking/cancellation happens solely from agent output.
- `entities::AuditEvent` entries for classification, policy decision, approval, external write, waiver/exception, and customer-facing send.

### Success state

The appointment has exactly one durable attendance/cancellation outcome, calendar capacity is accurately released or held, the customer/pet history reflects the policy-relevant event, any required deposit/rebooking restriction is represented as a typed decision, all customer-facing communication is approved before send, and future rebooking logic can explain the decision from semantic history rather than raw provider status strings.

### Failure and exception states

- Duplicate provider event: ignored or linked as duplicate without incrementing no-show/late-cancel counters twice.
- Conflicting facts: customer says cancelled with notice but provider says no-show; result is `DisputedRequiresManagerReview` and no penalty execution until manager approval.
- Missing appointment/customer/pet/location contract: produce `UnableToClassifyRequiresStaffReview`, not a default penalty.
- Missing communication consent: create internal task only; do not send a customer message.
- Outstanding deposit/payment ambiguity: return payment review requirement; grooming policy does not infer payment truth from text notes.
- Late cancellation caused by pet health/safety concern: route to care/medical review or manager review; do not auto-penalize as ordinary no-show.
- Provider write failure after approval: retain approved domain decision and audit the failed external command for retry/reconciliation.
- Manager waiver/exception: record waiver with typed approval actor/reason and preserve the original event for history analytics.

## 2. Domain types to add or refine

Recommended semantic paths for the later Rust card:

### Appointment identity and state

- `operations::grooming::AppointmentId`
  - Validated non-empty provider/domain ID or UUID wrapper. Provider-specific IDs are promoted at the adapter boundary.
- `operations::grooming::AppointmentState`
  - `Draft`, `HoldPendingApproval`, `Scheduled`, `Completed`, `Cancelled`, `LateCancelled`, `NoShow`, `DisputedRequiresReview`.
  - Invariant: only scheduled appointments can transition into cancellation/no-show outcomes; provider corrections must carry previous state and audit reason.
- `operations::grooming::attendance::Outcome`
  - `CancelledWithNotice { notice: NoticeGiven }`, `LateCancellation { notice: NoticeGiven }`, `NoShow { grace_period: GracePeriodMinutes }`, `ProviderCorrection { corrected_from: AppointmentState }`, `DisputedRequiresManagerReview`.
  - Invariant: no-show and late-cancel are not booleans; they are events with actor, time, evidence, and policy effect.

### Notice, grace, and counters

- `operations::grooming::NoticeHours`
  - Positive or explicit zero-allowed newtype depending on local policy; ordinary cancellation cutoff should use this instead of raw `u16`.
- `operations::grooming::NoticeGiven`
  - `BeforeCutoff { hours: NoticeHours }`, `InsideCutoff { hours: NoticeHours }`, `AfterStart`, `UnknownRequiresReview`.
- `operations::grooming::GracePeriodMinutes`
  - Bounded non-negative or positive scalar for marking no-shows after start time; if zero is allowed it should be explicit in the type constructor.
- `operations::grooming::NoShowCount` and `operations::grooming::LateCancellationCount`
  - Bounded counters used only for policy thresholds, never as generic integers.
- `operations::grooming::MissedAppointmentHistory`
  - Aggregates counts, most recent event, prior waivers, outstanding rebooking restriction, and dispute status for one customer/pet/location policy scope.

### Policy and decisions

- Refine `operations::grooming::NoShowPolicy`
  - Existing variants: `NoteHistoryOnly`, `RequireDepositForRebooking`, `ManagerReviewBeforeRebooking`.
  - Keep as top-level no-show outcome policy or split into `MissedAppointmentPolicy` when late cancellations share thresholds.
- Add `operations::grooming::CancellationPolicy`
  - Fields/variants should encode notice cutoff, late-cancel threshold, release slot/waitlist behavior, deposit effect, and review threshold.
  - Example variants: `NoPenaltyUntil { cutoff: NoticeHours }`, `LateCancellationCountsTowardDeposit`, `ReleaseSlotToWaitlist`, `ManagerReviewAfterRepeatedLateCancel { count: LateCancellationCount }`.
- `operations::grooming::MissedAppointmentPolicy`
  - Location contract combining no-show, cancellation, grace period, notice window, deposit/rebooking behavior, and exception gates.
- `operations::grooming::MissedAppointmentDecision`
  - `RecordOnly`, `ReleaseSlotAndDraftFollowUp`, `RequireDepositForRebooking`, `ManagerReviewRequired`, `DisputedRequiresManagerReview`, `PaymentReviewRequired`, `Disallowed`.
  - Invariant: decision carries typed reasons and follow-up actions; it is not inferred later from status strings.
- `operations::grooming::DepositRequirement`
  - `NotRequired`, `RequiredBeforeRebooking { policy: policy::Id }`, `AlreadySatisfied { reference: payment::PaymentReference }`, `WaiverRequiresManagerApproval`, `PaymentReviewRequired`.
- `operations::grooming::RebookingEligibility`
  - `Eligible`, `EligibleAfterDeposit`, `StaffHoldPendingApproval`, `ManagerReviewRequired`, `BlockedUntilPaymentResolved`.
- `operations::grooming::CalendarReleaseDecision`
  - `NoChange`, `ReleaseSlot`, `ReleaseAndOfferWaitlist`, `HoldForReschedulePendingApproval`, `ManualCalendarReview`.

### Evidence, reasons, and audit

- `operations::grooming::MissedAppointmentReason`
  - Customer-supplied or staff-selected reason category: `CustomerScheduleConflict`, `PetIllnessOrSafetyConcern`, `TransportationIssue`, `UnableToReachCustomer`, `ProviderError`, `Unknown`, `Other(extension::Label)`.
  - Invariant: reason category does not waive policy by itself; waiver is a manager-approved decision.
- `operations::grooming::AttendanceEvidence`
  - Validated bounded staff note/reference to provider event, call log, message, or check-in observation. Sensitive free text should be redacted in debug if it can contain customer/pet specifics.
- `operations::grooming::WaiverDecision`
  - `NotRequested`, `Denied`, `Approved { manager: entities::ManagerId, reason: policy::AutomationRationale }`.
- `operations::grooming::MissedAppointmentAuditTrail`
  - Typed collection of audit event IDs or audit facts for classification, policy evaluation, approval, provider write, customer message, and deposit action.

### Staff tasks and agent contracts

- Add grooming-specific `operations::StaffTaskKind` only when existing kinds are too vague:
  - `GroomingNoShowFollowUp { appointment_id: operations::grooming::AppointmentId }`
  - `GroomingCancellationReview { appointment_id: operations::grooming::AppointmentId }`
  - `GroomingDepositReview { customer_id: entities::CustomerId, pet_id: entities::PetId }`
  - `GroomingWaitlistFill { released_appointment_id: operations::grooming::AppointmentId }`
- `agents::grooming::NoShowCancellationInput`
  - Typed prompt packet input; contains history snapshot, policy snapshot, appointment facts, and allowed actions.
- `agents::grooming::NoShowCancellationRecommendation`
  - Recommendation with `MissedAppointmentDecision`, evidence, risk flags, confidence, and required review gates.
  - Invariant: agent output is advisory; deterministic policy must validate before tools execute.

## 3. Relationship map between types

### Entities

- `operations::grooming::Appointment` owns grooming appointment lifecycle facts once modeled. It references `entities::CustomerId`, `entities::PetId`, `entities::LocationId`, optional `entities::ReservationId`, service offering, schedule window, groomer assignment, state, and deposit requirement.
- `operations::grooming::MissedAppointmentEvent` is the immutable fact that an appointment was cancelled late, cancelled with notice, marked no-show, disputed, or provider-corrected.
- `operations::grooming::MissedAppointmentHistory` summarizes prior missed events for the customer/pet/location policy scope.

### Value objects

- `AppointmentId`, `NoticeHours`, `NoticeGiven`, `GracePeriodMinutes`, `NoShowCount`, `LateCancellationCount`, `AttendanceEvidence`, `MissedAppointmentReason`, `DepositRequirement`, and `RebookingEligibility` encode meaningful distinctions currently at risk of becoming raw strings, booleans, or integers.

### Policies

- `operations::grooming::MissedAppointmentPolicy` owns classification consequences.
- `operations::grooming::NoShowPolicyEngine` or `MissedAppointmentPolicyEngine` evaluates appointment facts, history, contract, and payment snapshot into a typed decision.
- `operations::grooming::AutomationPolicy` maps policy decisions and agent recommendations to `policy::AutomationLevel`/`policy::ReviewGate` requirements.
- `operations::grooming::ReminderPolicy` may consume the decision to draft approved follow-up timing, but it does not decide penalties.
- `operations::grooming::RebookingPolicy` consumes `RebookingEligibility` and deposit status before presenting future appointment options.

### Repositories/stores

- `operations::grooming::AppointmentRepository` loads appointment snapshots and appends domain attendance/cancellation outcomes.
- `operations::grooming::HistoryRepository` loads missed-appointment and service history by customer/pet/location.
- `operations::grooming::ContractRepository` loads the location contract including no-show/cancellation policy.
- `operations::grooming::CalendarRepository` reads current blocks and writes draft release/hold intents, not final provider mutations without approval.
- `payment::Repository` or payment tool port remains the owner for deposit truth and payment references.
- `tools::ReservationSystem`/future `tools::GroomingCalendarSystem` executes external draft/update commands after approval.

### Workflow events

- `workflow::WorkflowEventType::GroomingCancellationRequested` (new)
- `workflow::WorkflowEventType::GroomingNoShowObserved` (new)
- `workflow::WorkflowEventType::GroomingMissedAppointmentPolicyNeeded` (new)
- `workflow::WorkflowEventType::GroomingRebookingDepositRequired` (new)

Until those variants exist, use an extension-safe event label at the boundary, but promote into typed variants before core policy behavior grows.

### Staff tasks

- Customer follow-up: drafted after cancellation/no-show classification when communication is approved or staff review is needed.
- Manager review: repeated late cancels/no-shows, disputed facts, waiver/refund/deposit exception, or high-risk customer experience issue.
- Calendar review/waitlist fill: released slot can be offered safely only through staff-approved or provider-approved flow.
- Payment/deposit review: outstanding or ambiguous deposit state must be resolved by payment-owned contracts.

### Agent specs/tools

- Existing `grooming-rebooking` agent can be extended or paired with a specific `grooming-no-show-cancellation` workflow.
- Allowed tools: grooming appointment/history read, calendar availability read, policy read, task-create, draft-message, audit-read/write draft.
- Forbidden actions: mark provider no-show/cancelled, book/reschedule, waive/forfeit/refund deposit, charge payment, release slot, send message, or suppress disputed facts without approval.

## 4. Interaction contract

Rust-like pseudo-signatures are intentionally domain-shaped; exact lifetimes/async traits can be chosen later.

```rust
pub mod operations::grooming {
    pub struct MissedAppointmentEvent {
        pub appointment_id: AppointmentId,
        pub location_id: entities::LocationId,
        pub customer_id: entities::CustomerId,
        pub pet_id: entities::PetId,
        pub outcome: attendance::Outcome,
        pub reason: MissedAppointmentReason,
        pub evidence: AttendanceEvidence,
        pub observed_by: entities::ActorRef,
        pub observed_at: chrono::DateTime<chrono::Utc>,
    }

    pub struct MissedAppointmentPolicyEngine;

    impl MissedAppointmentPolicyEngine {
        pub fn classify(
            &self,
            appointment: &AppointmentSnapshot,
            observation: AttendanceObservation,
            contract: &Contract,
        ) -> Result<MissedAppointmentEvent>;

        pub fn decide(
            &self,
            event: &MissedAppointmentEvent,
            history: &MissedAppointmentHistory,
            payment: &payment::DepositStatusSnapshot,
            contract: &Contract,
        ) -> MissedAppointmentDecision;
    }

    pub trait AppointmentRepository {
        fn get(&self, id: AppointmentId) -> Result<AppointmentSnapshot>;
        fn append_missed_event(&self, event: MissedAppointmentEvent) -> Result<MissedAppointmentVersion>;
    }

    pub trait HistoryRepository {
        fn missed_appointment_history(
            &self,
            location: entities::LocationId,
            customer: entities::CustomerId,
            pet: entities::PetId,
        ) -> Result<MissedAppointmentHistory>;
    }

    pub trait ContractRepository {
        fn grooming_contract(&self, location: entities::LocationId) -> Result<Contract>;
    }

    pub trait CalendarRepository {
        fn draft_release_or_hold(
            &self,
            appointment: AppointmentId,
            decision: CalendarReleaseDecision,
            approval: Option<policy::ApprovalToken>,
        ) -> Result<CalendarCommandDraft>;
    }
}
```

Behavior ownership rules:

- `MissedAppointmentPolicyEngine::classify` owns classification because it compares appointment state, observation timing, notice window, grace period, and contract. Do not hide this in `helpers::is_no_show`.
- `MissedAppointmentPolicyEngine::decide` owns consequences because it combines classification, history, payment/deposit snapshot, and location policy.
- `payment` owns payment truth, deposit status, payment references, capture/refund/waiver execution, and payment errors. Grooming only returns requirements or review gates.
- `CalendarRepository` owns draft release/hold persistence; a provider tool owns final external mutation after approval.
- `ReminderPolicy`/workflow message drafting owns communication shape after the missed-appointment decision; the policy engine only says which review gate is required.
- `AutomationPolicy` owns whether a decision can create a draft, internal task, or approved tool command. It must reject LLM-only execution of booking/payment/message actions.

Precise flow contract:

1. Load appointment, contract, history, and deposit snapshot.
2. Classify the observation into `MissedAppointmentEvent` or `DisputedRequiresManagerReview`.
3. Idempotently append the event; duplicate provider events return existing version.
4. Evaluate `MissedAppointmentDecision`.
5. Convert decision into draft commands: staff task, customer-message draft, calendar draft, deposit requirement, audit events.
6. Execute external writes only when each draft has a typed approval token matching the review gate.

## 5. Review/approval contract

### Automation level

- Classification suggestion from provider timestamps and staff observations: `policy::AutomationLevel::InternalTaskOnly` until reviewed or until deterministic provider/status confidence is high enough for record-only append.
- Repeated no-show/late-cancel penalty, deposit requirement, or rebooking restriction: at least `policy::AutomationLevel::ManagerApprovalRequired` when customer-facing or payment-affecting.
- Drafting a customer follow-up message: `policy::AutomationLevel::DraftOnly` with `policy::ReviewGate::CustomerMessageApproval`.
- Calendar release/waitlist fill: draft/internal task by default; provider mutation requires staff/manager approval depending on conflict risk.
- Payment capture, deposit forfeiture, refund, or waiver: never executed by grooming automation; route through payment-owned workflow with `policy::ReviewGate::RefundOrDepositException` or manager approval.

### Review gates

- Staff review: first-time or low-risk cancellation/no-show recording, missing/ambiguous appointment details, calendar release confirmation, ordinary customer follow-up.
- Groomer review: disagreement about attendance, service preparation already performed, pet-specific handling/product context affecting cancellation reason.
- Manager review: repeated missed appointments, disputed facts, any waiver/exception, deposit forfeiture/refund, customer complaint, provider correction after a customer-facing message, or any attempted override of location policy.
- Customer-message approval: every customer-facing reminder, rebooking offer, deposit request, penalty explanation, apology, or dispute response.
- Payment/deposit exception review: any change to money/deposit state, waiver, forfeiture, refund, or ambiguity in deposit status.

### Audit trail

Every material step writes or drafts an `entities::AuditEvent` with typed metadata:

- classification source and actor;
- policy snapshot/contract ID used;
- history counts considered;
- decision and required review gates;
- approval token/manager actor when present;
- external provider command ID/result;
- customer-message draft/send ID;
- deposit/payment reference or review task ID.

Do not store raw provider JSON or unbounded customer-sensitive text in domain audit metadata. Store external provider/id refs and bounded semantic values.

### Customer/member-facing boundaries

Automation may draft but not send:

- cancellation/no-show confirmations;
- deposit/rebooking requirements;
- penalty/waiver/refund explanations;
- rebooking offers;
- apologies or dispute responses.

Automation may not:

- mark a customer as no-show in the provider system without approved source facts;
- charge, forfeit, waive, or refund a deposit;
- block a customer from rebooking without a typed policy decision and review gate;
- hide disputed facts or concerning groomer notes;
- modify boarding/daycare reservations when grooming was attached as an add-on.

## 6. Test contracts

Domain tests should read like semantic glossary entries:

1. `grooming_cancellation_with_sufficient_notice_records_cancelled_with_notice_without_deposit_requirement`
   - Given a scheduled appointment and notice before the cutoff, policy returns `RecordOnly` or `ReleaseSlotAndDraftFollowUp` with `DepositRequirement::NotRequired`.
2. `grooming_late_cancellation_inside_cutoff_counts_separately_from_no_show`
   - Late cancellation increments `LateCancellationCount`, not `NoShowCount`, unless the location contract explicitly maps both into a shared missed-appointment threshold.
3. `grooming_no_show_after_grace_period_requires_deposit_for_rebooking_when_contract_says_so`
   - Current `Contract::standard_petsuites()` behavior maps no-show to deposit requirement before rebooking.
4. `grooming_repeated_late_cancellations_route_to_manager_review_before_rebooking_restriction`
   - Threshold policy produces `ManagerReviewRequired`; it does not silently block customer booking from raw counts.
5. `grooming_disputed_no_show_does_not_forfeit_or_require_deposit_without_manager_approval`
   - Conflicting customer/provider facts produce disputed review and no payment-affecting execution.
6. `grooming_no_show_policy_is_idempotent_for_duplicate_provider_events`
   - Replaying the same provider no-show event returns existing missed-event version and does not double-count history.
7. `grooming_cancellation_due_to_pet_health_routes_to_care_or_manager_review_before_penalty`
   - Pet illness/safety reason triggers review gate instead of ordinary auto-penalty.
8. `grooming_calendar_release_is_draft_until_staff_or_manager_approval`
   - Decision can create a draft release/waitlist command, but provider mutation requires approval token.
9. `grooming_deposit_requirement_references_payment_contract_without_capturing_payment`
   - Grooming returns `DepositRequirement`; payment tool/repository owns capture/status/reference changes.
10. `grooming_customer_message_about_no_show_requires_customer_message_approval`
    - Agent/policy can draft a message but cannot produce approved send without review.
11. `grooming_manager_waiver_preserves_original_no_show_event_and_records_approval_actor`
    - Waiver changes rebooking/deposit consequence, not the immutable event history.
12. `grooming_missed_appointment_audit_trail_records_policy_snapshot_actor_and_external_refs`
    - Audit metadata uses typed keys/values and provider refs, not raw unbounded payloads.
13. `grooming_rebooking_policy_respects_deposit_requirement_before_presenting_schedulable_slot`
    - Future appointment plan is `StaffHoldPendingApproval` or `EligibleAfterDeposit`, not freely schedulable.
14. `grooming_no_show_cancellation_agent_recommendation_is_validated_by_deterministic_policy_before_tool_execution`
    - Agent output without approval token maps to draft/internal task only.

Storage/boundary tests:

1. `grooming_missed_appointment_record_rejects_unknown_status_without_review_required_extension`
2. `grooming_provider_no_show_payload_promotes_to_typed_appointment_id_actor_and_outcome`
3. `grooming_missed_appointment_history_record_preserves_late_cancel_and_no_show_counts_separately`
4. `grooming_audit_metadata_redacts_or_refs_sensitive_customer_reason_text`
5. `grooming_external_calendar_command_cannot_be_confirmed_from_draft_decision_without_approval_token`

Agent/workflow tests:

1. `grooming_no_show_cancellation_agent_forbids_booking_payment_waiver_and_send_actions`
2. `grooming_no_show_cancellation_prompt_packet_contains_policy_snapshot_history_and_allowed_actions`
3. `grooming_rebooking_agent_marks_deposit_required_customer_followup_as_customer_message_review_required`
4. `manager_daily_brief_surfaces_repeated_grooming_no_shows_as_manager_attention_required`

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Short-term: extend `operations::grooming` with missed-appointment value types, policy/decision enums, and contract fields.
  - Medium-term: consider splitting `operations::grooming` into a directory module once appointment, cancellation, rebooking, reminders, and history grow beyond the current inline module.
- `domain/src/entities.rs`
  - Add audit subject/action extensions only if generic extension labels are insufficient; prefer typed grooming appointment subject once `AppointmentId` exists.
- `domain/src/policy.rs`
  - Reuse `ReviewGate::{ManagerApproval, CustomerMessageApproval, RefundOrDepositException}` and `AutomationLevel`; add a narrower review reason only if needed.
- `domain/src/workflow.rs`
  - Add grooming missed-appointment workflow event variants or a typed extension path.
- `domain/src/agents.rs`
  - Add or refine grooming no-show/cancellation agent spec and output schema names; current `grooming-rebooking` spec already establishes key forbidden actions.
- `domain/src/tools.rs`
  - Add grooming calendar/status/deposit draft command ports only after domain decisions exist; keep provider writes approval-gated.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add standard contract assertions for cancellation/no-show policy once fields exist.
- `domain/tests/domain_quality_patterns.rs`
  - Add semantic quality tests for newtypes, audit metadata, agent packets, and review gates.
- Storage tests/modules in later storage card
  - Add records/codecs for missed appointment events/history and reject cross-status/raw-string shapes.

### Migration/refactor risks

- Current `operations::grooming::NoShowPolicy` is too narrow for late cancellations. Avoid overloading it until it obscures the distinction; introduce `MissedAppointmentPolicy` or `CancellationPolicy` when code needs both.
- Current `operations::grooming::CadenceWeeks` and top-level `operations::CadenceWeeks` duplicate semantics. Do not introduce a third cadence/count/notice scalar; choose the grooming-owned path for grooming-specific policy.
- Do not attach deposit capture/waiver/refund behavior to grooming types. Grooming policy may require or reference deposit state, but payment remains the owner of money movement.
- Do not mutate reservation status/add-ons when the grooming appointment is attached to boarding/daycare. Return draft commands or typed reservation references.
- Do not use generic `CustomerFollowUp`/`DocumentReview` tasks if grooming-specific no-show/deposit review semantics are needed at call sites; add truthful `StaffTaskKind` variants.
- Idempotency is critical because provider webhooks/status syncs can replay; missed-event storage needs source event IDs or deterministic dedupe keys.
- Customer reason text can contain sensitive medical/personal details; model categories and bounded/redacted evidence rather than raw logs in core domain/audit.
- Approval tokens must be bound to action kind and policy snapshot; a manager approval for a message should not authorize deposit forfeiture or provider status mutation.

### Dependencies on other implications/domain surfaces

- Rebooking cadence/history implication: no-show/late-cancel history directly constrains whether a rebooking recommendation can become schedulable.
- Reminder/customer communication implication: cancellation/no-show follow-up messages must use the same consent and customer-message approval boundary as grooming reminders.
- Calendar/scheduling implication: release slot, draft hold, waitlist fill, and reschedule suggestions depend on groomer availability and buffer policy.
- Service history implication: missed-appointment events are operational history but should remain distinct from completed grooming service/style history.
- Payment/deposit foundation: deposit status, payment references, waiver/refund/forfeiture, and collection attempts require payment-owned contracts.
- Audit/workflow foundation: durable audit events and workflow event types are needed before provider writes and member-facing sends are trusted.

Implementation entry rule for the later code card: write the failing semantic test for the smallest policy slice first, likely `grooming_no_show_after_grace_period_requires_deposit_for_rebooking_when_contract_says_so`; then add the minimal `MissedAppointmentEvent`, `MissedAppointmentDecision`, and policy engine surface. Do not start with provider adapters or agent prompts before the core decision vocabulary exists.
