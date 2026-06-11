# Boarding implication 08: Payment/deposit handling

Purpose: define the operational and domain contract for Boarding payment/deposit handling. This is a modeling artifact for later Rust implementation; it does not authorize live charges, refunds, waivers, forfeitures, reservation confirmations, or customer-facing payment messages.

Source context:

- Boarding service map: `docs/domain/petsuites/boarding/service-domain-map.md`.
- Current Rust surfaces: `domain/src/operations.rs`, `domain/src/payment/mod.rs`, `domain/src/money/mod.rs`, `domain/src/entities.rs`, `domain/src/workflow.rs`, `domain/src/tools.rs`.
- Current tests: `domain/tests/petsuites_core_service_contracts.rs`, `domain/tests/domain_quality_patterns.rs`.

Assumptions:

- Exact dollar amounts, holiday deadlines, refund windows, payment processor behavior, and local exception rules are location/provider policy data, not hard-coded Boarding constants.
- Boarding owns when a deposit/payment obligation is required for a stay and what operational review gates follow from that obligation. `money` owns amounts/currency. `payment` owns actual payment references/status. `entities::Reservation` owns the cross-service reservation lifecycle and stores the actual deposit projection.
- In the safest default model, a boarding stay with `PaymentTiming::DueAtBooking` and an unpaid required deposit is not ready for confirmation; the system may draft an internal collection task or customer-message draft, but cannot collect money or confirm the booking without approval and an executing payment boundary.

## 1. Operational story

### Trigger

Payment/deposit handling begins whenever a boarding stay is quoted, requested, modified, confirmed, checked in, checked out, cancelled, no-showed, or reviewed for an exception. Common triggers:

- A customer submits or staff creates a boarding request.
- A location contract/season policy says the requested dates require a deposit.
- Holiday/peak demand changes minimum-stay, deposit, cancellation, or refund handling.
- A reservation is ready for confirmation but the required deposit is missing, failed, or ambiguous.
- A customer requests cancellation/change/refund/waiver, especially inside the cancellation notice window.
- Check-in/check-out exposes an unpaid balance, missing deposit reference, failed payment, or manager exception.
- An agent detects deposit/payment risk during a pre-arrival checklist or daily boarding brief.

### Actors

- Customer/member: requests boarding, pays deposit/balance, requests cancellation/refund/waiver, receives approved messages only.
- Front desk/staff: reviews payment/deposit status, drafts or sends approved reminders, collects payment through approved tools, verifies processor references, prepares check-in/check-out tasks.
- Manager: approves deposit waivers, refunds, forfeitures, late-cancellation exceptions, policy overrides, ambiguous payment reconciliation, and sensitive customer explanations.
- Payment provider/tool adapter: executes actual charge/refund/void operations only from approved tool commands; returns processor references/status.
- Boarding policy evaluator: deterministic domain owner for deposit requirement, due timing, cancellation outcome, and review gates.
- Agent/workflow: may read/summarize/detect/draft/create internal tasks; must not charge, refund, waive, forfeit, confirm, cancel, or send payment-sensitive messages autonomously.

### Inputs

- `entities::LocationId`, location timezone, `entities::LocationPolicyRefs.deposit_policy_id`, and Boarding contract version.
- `operations::boarding::Contract.deposit`, `Contract.payment`, `Contract.cancellation`, minimum-stay and season policy.
- Proposed `operations::boarding::StayRequest` or existing `entities::Reservation` with service `ServiceKind::Boarding`.
- Date range, stay nights, peak/holiday period, accommodation/add-ons, pet/customer ids, and reservation source.
- `money::Money` amount(s), currency, taxes/fees if later modeled outside the deposit rule.
- Current `payment::Deposit` projection: amount, `payment::DepositStatus`, payment reference, refundable-until timestamp.
- Existing `workflow::WorkflowEvent` / `operations::StaffTask` / audit history related to collection, refund, waiver, forfeiture, cancellation, and manager approvals.

### Decisions

- Is a deposit required for this stay?
- If required, what amount, due timing, refundability, and due deadline apply to this stay snapshot?
- Is the current deposit state sufficient for the requested transition: quote, offer, confirm, check in, check out, cancel, refund, or waive?
- Does a failed/missing/ambiguous payment reference require internal staff review, manager review, or provider reconciliation?
- Can cancellation proceed without penalty, with deposit forfeiture, with refund, or only after manager review?
- Should the system create an internal staff task, draft a customer reminder, block confirmation/check-in/check-out, or escalate to a manager?
- Which audit records and workflow events must be persisted before tool execution or customer-facing communication?

### Outputs

- `operations::boarding::deposit::Decision` explaining requirement and due state.
- `operations::boarding::payment::Readiness` explaining whether the stay can move to the requested lifecycle phase.
- `operations::boarding::cancellation::Outcome` for cancellation/change requests involving deposits.
- `workflow::RecommendedAction` values for internal tasks, payment collection draft, refund/waiver/forfeit review, reservation-update draft, or customer-message draft.
- `operations::StaffTask` drafts assigned to `FrontDesk` or `Manager` with semantic reason/source.
- `entities::AuditEvent` / `workflow::WorkflowEvent` records preserving the decision, actor, inputs, review gate, and approval state.
- Approved tool command drafts, never raw payment mutations from the domain model.

### Success state

A boarding payment/deposit interaction is successful when:

- The stay has an immutable policy snapshot for deposit amount, due timing, refundability, cancellation rule, and local/season context.
- The current `payment::DepositStatus` truthfully matches the reservation phase.
- Confirmation/check-in/check-out/cancellation readiness is expressed as a typed decision, not an implicit boolean or status string.
- Any required customer-facing payment communication is either approved and sent through a boundary tool or remains a draft with review reason.
- Any charge/refund/waiver/forfeit action has the required staff/manager approval, a payment reference when applicable, and an audit trail.

### Failure and exception states

- Missing deposit rule for a location/season where policy requires one: block confirmation and create manager review.
- Required deposit unpaid at `DueAtBooking`: not ready to confirm; create collection task/reminder draft.
- Required deposit unpaid at `DueAtCheckIn`: check-in cannot complete without staff collection or manager exception.
- Required balance unpaid at `DueAtCheckout`: checkout cannot complete without staff collection or manager exception.
- Payment failed/declined/processor reference missing: staff reconciliation task; no automatic retry unless a payment tool contract later makes retry approval explicit.
- Amount/currency mismatch between Boarding rule and payment projection: manager/payment reconciliation review.
- Late holiday cancellation: manager review or typed forfeiture path; no automatic refund/waiver.
- Deposit waiver/refund/forfeit requested: manager approval required before any tool command or customer-facing explanation.
- Customer disputes payment, safety/incident content intersects with payment, or legal/regulatory signal appears: manager/legal review; agent remains draft-only.

## 2. Domain types to add or refine

### `operations::boarding::deposit`

- `operations::boarding::deposit::Policy`
  - Owns Boarding deposit requirement evaluation for a stay request/reservation snapshot.
  - Composes contract defaults, season/holiday policy, cancellation policy, customer/reservation history if available, and manager override evidence.
- `operations::boarding::deposit::Rule`
  - Refine or re-export current `operations::boarding::DepositRule`.
  - Variants should remain semantic: `NotRequired`, `Required { amount: money::Money }`, later `RequiredBySeason { amount, period }`, `ManagerReviewRequired { reason }` if local rules cannot be evaluated deterministically.
- `operations::boarding::deposit::Decision`
  - `NotRequired`
  - `Required { amount: money::Money, due: deposit::Due, refundable_until: Option<DateTime<Utc>>, reason: deposit::RequirementReason }`
  - `AlreadySatisfied { deposit: payment::Deposit }`
  - `CollectionRequired { amount, due, current_status: payment::DepositStatus, reason }`
  - `ManagerReviewRequired { reason: deposit::ReviewReason }`
- `operations::boarding::deposit::Due`
  - Semantic due timing: `AtBooking`, `AtCheckIn`, `AtCheckout`, `ByLocalDeadline { at: DateTime<Utc> }`.
  - Refines current `PaymentTiming`; `PaymentTiming` can remain contract-level, while `deposit::Due` is a per-stay evaluated due.
- `operations::boarding::deposit::RequirementReason`
  - `StandardBoardingPolicy`, `HolidayPeak`, `LocalEvent`, `CancellationRisk`, `MultiPetStay`, `ManagerOverride`, `ProviderPolicy`.
- `operations::boarding::deposit::ReviewReason`
  - `MissingPolicy`, `AmountMismatch`, `CurrencyMismatch`, `PaymentReferenceMissing`, `PaymentFailed`, `WaiverRequested`, `RefundRequested`, `ForfeitureRequested`, `LateCancellation`, `ManagerOverrideRequested`.
- `operations::boarding::deposit::Snapshot`
  - Immutable per-reservation policy snapshot: rule, amount, due, refundability, cancellation linkage, evaluated at, policy refs/version.

### `operations::boarding::payment`

- `operations::boarding::payment::Policy`
  - Owns readiness checks for reservation transitions that depend on deposit/payment state.
  - Does not execute money movement.
- `operations::boarding::payment::Readiness`
  - `Ready { evidence: payment::Evidence }`
  - `Blocked { reason: payment::BlockReason, required_task: task::Kind }`
  - `ManagerReviewRequired { reason: deposit::ReviewReason }`
  - `DraftCustomerReminder { reason: payment::ReminderReason, review_gate: policy::ReviewGate }`
- `operations::boarding::payment::Evidence`
  - Typed evidence that deposit/payment obligation has been satisfied, waived by manager, not required, or deferred by approved policy.
- `operations::boarding::payment::BlockReason`
  - `DepositRequired`, `DepositFailed`, `PaymentReferenceAmbiguous`, `BalanceDueAtCheckout`, `PolicySnapshotMissing`, `ApprovalMissing`.
- `operations::boarding::payment::Repository`
  - Boarding-facing port for reading payment/deposit projection and drafting collection/refund/waiver/forfeit requests.
  - It returns semantic domain values; provider JSON stays in adapter/storage.

### `operations::boarding::cancellation`

- `operations::boarding::cancellation::Policy`
  - Refines current `CancellationPolicy` by evaluating request timestamp, stay state, peak period, deposit status, and manager approvals.
- `operations::boarding::cancellation::Outcome`
  - `AllowedNoPenalty`
  - `AllowedRefundDeposit { amount: money::Money }`
  - `AllowedForfeitDeposit { amount: money::Money }`
  - `ManagerReviewRequired { reason: cancellation::ReviewReason }`
  - `DeniedAfterCheckIn`
- `operations::boarding::cancellation::ReviewReason`
  - `InsideNoticeWindow`, `HolidayPeakPolicy`, `RefundRequested`, `ForfeitureDisputed`, `AlreadyCheckedIn`, `IncidentOrSafetyRelated`, `PolicyAmbiguous`.

### Shared/refined supporting types

- `money::Money`, `money::MinorUnits`, `money::Currency`
  - Keep amount/currency ownership here. Do not create Boarding-specific cents/currency primitives.
- `payment::Deposit`, `payment::DepositStatus`, `payment::PaymentReference`
  - Keep actual payment state here. Consider adding transition methods for refund/failed/waived if later cards need them, but manager approval should be represented before calling them.
- `entities::Reservation.deposit`
  - Store actual deposit projection. Do not store Boarding policy-only state as a loose `Option<Deposit>` without a policy snapshot once behavior grows.
- `operations::boarding::task::Kind`
  - Add `DepositCollection`, `PaymentReconciliation`, `RefundReview`, `WaiverReview`, `ForfeitureReview`, `CheckoutBalanceReview`, `CustomerReminderReview` and map to `operations::StaffTaskKind`.
- `operations::boarding::agent::ApprovalPolicy`
  - Add explicit payment/deposit action classification.

## 3. Relationship map

### Entities

- `entities::Reservation`
  - Cross-service lifecycle aggregate; carries `service: ServiceKind::Boarding`, `status`, `deposit`, requested add-ons, hard stops, and timestamps.
  - Boarding payment policy evaluates proposed lifecycle changes but should not directly mutate live reservation state.
- `entities::CustomerId`
  - Paying/communicating customer reference. Contact details and preferences remain in customer/profile modules.
- `entities::PetId`
  - Payment policy usually does not depend on pet details except through stay/add-on/care-risk context; cancellation exceptions may mention care/safety events.
- `entities::LocationId`
  - Selects Boarding contract, deposit policy refs, timezone, local holiday periods, and provider execution boundaries.

### Value objects

- `money::Money` / `MinorUnits` / `Currency`: amount truth.
- `payment::PaymentReference`: non-empty provider/reference identity for paid/refunded payment events.
- `operations::boarding::deposit::Due`: per-stay due timing/deadline.
- `operations::boarding::deposit::Snapshot`: immutable evaluated policy facts.
- `operations::boarding::stay::DateRange` and `stay::Nights`: stay length and local date facts that influence deposit/cancellation.
- `operations::boarding::season::Period`: holiday/peak/local-event facts that influence requirement/refund windows.

### Policies

- `boarding::deposit::Policy`: deposit requirement and due-state evaluation.
- `boarding::payment::Policy`: reservation-phase readiness based on deposit/payment evidence.
- `boarding::cancellation::Policy`: cancellation/refund/forfeit outcome.
- `boarding::agent::ApprovalPolicy`: automation/review classification for payment/deposit actions.
- `policy::ReviewGate` / `policy::AutomationLevel`: cross-domain review envelope consumed by workflow/tool layers.

### Repositories and stores

- `boarding::Repository`: contract snapshot by location/version.
- `boarding::payment::Repository`: read deposit state; draft collection/refund/waiver/forfeit commands; never silently execute.
- `boarding::reservation::Repository`: read Boarding reservation projections and propose updates through workflow/tools.
- `storage::operations`: persist contract/policy snapshots; later `storage::boarding::payment` may persist evaluated snapshots and decision history.
- Payment provider adapter: boundary execution store/tool; translates approved command drafts into processor calls and returns `payment::PaymentReference`/status.

### Workflow events

- `DepositEvaluated`
- `DepositCollectionTaskCreated`
- `PaymentReminderDrafted`
- `DepositMarkedPaid`
- `PaymentReferenceReconciliationRequired`
- `DepositWaiverRequested`
- `DepositWaiverApproved`
- `RefundRequested`
- `RefundApproved`
- `DepositForfeitureRecommended`
- `CancellationPaymentOutcomeEvaluated`

These can initially be represented with `workflow::WorkflowEvent` plus semantic metadata. Later code should promote stable variants if they become first-class workflow concepts.

### Staff tasks

- Front desk: collect deposit, verify payment reference, send approved reminder, reconcile failed/ambiguous payment, collect checkout balance.
- Manager: approve waiver, refund, forfeiture, late-cancellation exception, amount mismatch, missing/ambiguous local policy.
- Agent-created internal tasks must preserve Boarding source context: reservation id, customer id, location id, due deadline, current status, policy reason, review gate.

### Agent specs/tools

- Agent specs:
  - `boarding_prearrival_payment_checklist_agent`: reads upcoming arrivals; drafts internal tasks for missing/failed deposits.
  - `boarding_cancellation_deposit_review_agent`: summarizes cancellation/refund/forfeiture evidence for manager review.
  - `boarding_checkout_balance_agent`: drafts checkout balance review tasks.
- Tools:
  - `tools::PaymentCollectionDraft`, `tools::RefundDraft`, `tools::ReservationUpdateDraft`, `tools::CustomerMessageDraft` should be approved artifacts, not direct domain mutations.
  - If these tool types do not exist yet, later code should add narrow command-draft types rather than stuffing payment actions into generic notes.

## 4. Interaction contract

Rust-like pseudo-signatures below show ownership; exact names may change during implementation, but behavior should remain on truthful owners.

```rust
impl operations::boarding::deposit::Policy {
    pub fn evaluate(
        &self,
        contract: &operations::boarding::Contract,
        stay: &operations::boarding::StayRequest,
        season: Option<&operations::boarding::season::Period>,
        existing: Option<&payment::Deposit>,
    ) -> operations::boarding::deposit::Decision;
}
```

`deposit::Policy` owns the requirement decision. It should not call payment tools or mutate reservations.

```rust
impl operations::boarding::payment::Policy {
    pub fn readiness_for_confirmation(
        &self,
        reservation: &entities::Reservation,
        deposit: operations::boarding::deposit::Decision,
    ) -> operations::boarding::payment::Readiness;

    pub fn readiness_for_check_in(
        &self,
        reservation: &entities::Reservation,
        deposit: &payment::Deposit,
    ) -> operations::boarding::payment::Readiness;

    pub fn readiness_for_check_out(
        &self,
        reservation: &entities::Reservation,
        balance: operations::boarding::payment::BalanceState,
    ) -> operations::boarding::payment::Readiness;
}
```

`payment::Policy` owns lifecycle readiness. It should return typed blockers, review gates, and task recommendations rather than booleans.

```rust
impl operations::boarding::cancellation::Policy {
    pub fn evaluate(
        &self,
        reservation: &entities::Reservation,
        requested_at: DateTime<Utc>,
        deposit: Option<&payment::Deposit>,
        season: Option<&operations::boarding::season::Period>,
    ) -> operations::boarding::cancellation::Outcome;
}
```

`cancellation::Policy` owns refund/forfeit/no-penalty recommendations. Actual refunds/forfeitures remain approved tool commands.

```rust
trait operations::boarding::payment::Repository {
    fn load_deposit(
        &self,
        reservation: entities::ReservationId,
    ) -> payment::Result<Option<payment::Deposit>>;

    fn draft_collection(
        &self,
        request: operations::boarding::payment::CollectionRequest,
    ) -> workflow::Result<workflow::RecommendedAction>;

    fn draft_refund_or_waiver_review(
        &self,
        request: operations::boarding::payment::ExceptionRequest,
    ) -> workflow::Result<workflow::RecommendedAction>;
}
```

The repository/port owns persistence and command drafting, not policy evaluation. It should not hide provider execution behind a vague `process_payment` helper.

```rust
impl operations::boarding::workflow::Planner {
    pub fn plan_payment_tasks(
        &self,
        reservation: &entities::Reservation,
        readiness: operations::boarding::payment::Readiness,
    ) -> Vec<operations::StaffTask>;
}
```

`workflow::Planner` owns mapping decisions into staff tasks. Boarding-specific task kinds should remain visible before mapping to generic `operations::StaffTaskKind`.

```rust
impl operations::boarding::agent::ApprovalPolicy {
    pub fn classify_payment_action(
        &self,
        action: &operations::boarding::payment::ProposedAction,
    ) -> policy::AutomationLevel;
}
```

Approval policy owns automation level. Tool adapters must require an explicit approved action for charge/refund/waiver/forfeit/customer-message execution.

Behavior placement rules:

- `payment::Deposit::requires_collection()` can remain a local status helper because it asks a fact about the deposit value.
- `boarding::Contract::requires_deposit_collection()` is only a contract-default helper; per-reservation due state belongs to `boarding::deposit::Policy`.
- Customer reminder generation belongs to workflow/message-draft planning, not to `payment::Deposit`.
- Refund/forfeit evaluation belongs to `boarding::cancellation::Policy`; execution belongs to payment tool adapters after approval.

## 5. Review/approval contract

### Automation level

Safe/internal-only:

- Read reservation/deposit status.
- Evaluate deterministic deposit/cancellation/payment readiness policies.
- Summarize deposit risk for daily brief/pre-arrival checklist.
- Create internal task drafts for staff/manager review.
- Draft, but not send, customer reminders or explanations.

Staff approval required:

- Sending routine customer reminders about unpaid deposit/balance.
- Marking payment evidence as reconciled when the provider reference is present but not auto-verified.
- Completing check-in/check-out after staff collection.
- Applying non-sensitive reservation status recommendations after payment is verified.

Manager approval required:

- Waiving a deposit.
- Refunding a deposit.
- Forfeiting a deposit, especially for late/holiday cancellations.
- Overriding due timing, local policy, amount mismatch, or holiday/peak rules.
- Sending sensitive explanations about denial, cancellation penalty, refund refusal, incident-linked payment, dispute, or legal/regulatory matters.

Never fully automated/member-facing without explicit approval:

- Charging a card, refunding a payment, voiding a payment, waiving a deposit, or forfeiting a deposit.
- Confirming/cancelling/rejecting/modifying a live reservation based on payment state.
- Sending customer-facing payment/cancellation/refund policy messages.
- Hiding failed payment, unresolved deposit, or unresolved manager review from check-in/check-out readiness.

### Review gates

Use explicit review gates instead of comments or booleans:

- `ReviewGate::PaymentCollectionApproval` if added later, or a semantically equivalent local gate under `boarding::payment`.
- `ReviewGate::RefundOrDepositException` for waiver/refund/forfeit/late-cancellation exceptions.
- `ReviewGate::CustomerMessageApproval` for any payment/cancellation reminder or explanation.
- `ReviewGate::ManagerPolicyOverride` for local/holiday/deadline ambiguity.
- `ReviewGate::PaymentReconciliation` for failed/missing/mismatched references.

### Audit trail

Every payment/deposit decision should be auditable with:

- Actor: customer, staff, manager, system, or agent.
- Subject: reservation, customer, location, workflow event, and payment reference when available.
- Policy evidence: contract version, deposit policy ref, season period, due timing, amount/currency, current status.
- Decision: required/not required/collection required/waived/refund recommended/forfeit recommended/manager review.
- Review gate and approval state.
- Tool command id/reference and processor reference for executed boundary actions.

### Customer/member-facing boundary

Customer-facing outputs must be drafts until approved. Drafts should carry:

- Contact channel from `entities::Customer.preferred_contact`.
- Reason and approved template/policy citation.
- What action the customer is asked to take.
- What must not be promised: final booking confirmation, refund guarantee, waiver approval, live availability, or exception outcome.

## 6. Test contracts

Future implementation cards should add semantic tests with names like these:

- `boarding_deposit_policy_requires_due_at_booking_collection_before_confirmation`
  - A required due-at-booking deposit with status `Required` or `Failed` returns collection required and blocks confirmation.
- `boarding_deposit_policy_treats_paid_deposit_with_reference_as_satisfied`
  - A paid deposit with non-empty `payment::PaymentReference` produces satisfied evidence.
- `boarding_deposit_policy_routes_missing_payment_reference_to_reconciliation_review`
  - A paid/claimed-paid deposit without provider evidence is not silently accepted.
- `boarding_deposit_decision_preserves_amount_currency_due_reason_and_policy_snapshot`
  - Decision carries `money::Money`, `deposit::Due`, requirement reason, location/policy version evidence.
- `boarding_deposit_rule_does_not_duplicate_money_primitives`
  - Boarding deposit rule composes `money::Money` instead of raw cents/currency strings.
- `boarding_confirmation_readiness_blocks_unpaid_due_at_booking_deposit`
  - Reservation confirmation readiness produces a typed blocker and staff task, not a boolean.
- `boarding_check_in_readiness_blocks_unpaid_due_at_checkin_deposit_without_manager_exception`
  - Check-in requires collection or approved manager exception.
- `boarding_checkout_readiness_blocks_balance_due_without_staff_collection_or_manager_exception`
  - Checkout readiness includes balance/deposit review.
- `boarding_late_holiday_cancellation_routes_to_manager_review_or_typed_forfeiture`
  - Holiday/peak late cancellation cannot auto-refund/waive.
- `boarding_refund_waiver_and_forfeiture_actions_require_manager_approval`
  - Approval policy maps exception actions to manager review.
- `boarding_agent_can_create_internal_deposit_task_but_cannot_charge_refund_or_waive`
  - Agent classification allows internal task draft only.
- `boarding_customer_payment_message_remains_draft_until_staff_or_manager_approved`
  - Message tool receives no sendable command without review clearance.
- `boarding_payment_reconciliation_task_preserves_reservation_customer_location_and_policy_context`
  - Staff task source/context remains typed and traceable.
- `boarding_cancellation_outcome_records_refund_forfeit_or_review_reason_as_enum`
  - Cancellation result uses semantic outcome variants, not raw strings.
- `boarding_payment_audit_event_records_actor_subject_decision_review_gate_and_payment_reference`
  - Audit trail is reconstructable without parsing free text.

## 7. Integration notes for later serialized Rust code cards

### Files likely touched

- `domain/src/operations.rs`
  - Expand `pub mod boarding` with child modules or nested types for `deposit`, `payment`, `cancellation`, `task`, and possibly `workflow`.
  - Preserve `operations::boarding::Contract` and `standard_petsuites()` as the public contract root.
  - Consider re-exporting current `DepositRule`, `PaymentTiming`, `CancellationPolicy`, and `CancellationPenalty` through semantic child modules instead of creating parallel types immediately.
- `domain/src/payment/mod.rs`
  - May need richer transition methods/status variants for refunded/failed/reconciled states, but avoid putting Boarding-specific policy here.
- `domain/src/money/mod.rs`
  - Likely unchanged except tests should assert Boarding composes `Money`.
- `domain/src/entities.rs`
  - `Reservation.deposit` already exists. Later cards may add a policy snapshot/projection field or keep it in Boarding-specific storage/workflow events.
  - `HardStop::DepositRequired` already exists and can be mapped from `payment::Readiness::Blocked`.
- `domain/src/workflow.rs`, `domain/src/tools.rs`, `domain/src/agents.rs`, `domain/src/policy.rs`
  - Add or refine review gates, recommended actions, tool command drafts, and agent approval classification for payment/deposit actions.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add contract-level tests for deposit/payment invariants.
- `domain/tests/domain_quality_patterns.rs`
  - Add semantic-pattern tests if module path/value-object rules are checked there.
- Storage layer, if present in later cards:
  - Update `storage::operations::CoreServiceContractsRecord` codecs if contract fields move/refactor.
  - Add migration/roundtrip tests for contract snapshots and payment/deposit decision snapshots.

### Migration/refactor risks

- Existing storage roundtrips may depend on the current flat `operations::boarding::Contract` fields. Prefer additive child-module re-exports and compatibility codecs before changing serialized names.
- Current `DepositRule::Required { amount }` is contract-level policy, while `payment::Deposit` is reservation-level state. Mixing them would create false truths; keep requirement and status separate.
- Current `PaymentTiming` is coarse and contract-level. A per-stay `deposit::Due` should include local deadlines without breaking existing contract tests.
- `payment::DepositStatus::WaivedByManager` currently encodes an approved result but not the approval evidence. Later cards must pair it with workflow/audit records or an approved exception value.
- `entities::Reservation.deposit: Option<Deposit>` can represent no deposit, but absence is ambiguous unless paired with a `deposit::Decision::NotRequired` or policy snapshot.
- Avoid generic `process_payment`, `handle_deposit`, or raw `String` status helpers. Payment/deposit behavior has policy, lifecycle, approval, and audit meaning.
- Do not let an agent approval shortcut leak into payment tools. Tool commands must prove approval explicitly.

### Dependencies on other implications

- Capacity/holiday/peak-period implications: deposit requirement and cancellation outcomes depend on season and demand periods.
- Reservation lifecycle/check-in/check-out implications: payment readiness gates reservation phase transitions.
- Care/incident/customer-communication implications: sensitive payment explanations may require manager/legal review when care, safety, or incident facts are involved.
- Pawgress/report/customer-message implications: payment reminders and cancellation explanations share customer-facing approval boundaries.
- Staff task/handoff implications: unresolved deposit/payment blockers must appear in pre-arrival, check-in, checkout, and shift handoff tasks.

### Recommended implementation slice

1. Add `boarding::deposit::Decision`, `Due`, `RequirementReason`, `ReviewReason`, and a small deterministic `Policy` using existing `Contract`, `DepositRule`, `PaymentTiming`, `payment::Deposit`, and `payment::DepositStatus`.
2. Add `boarding::payment::Readiness` and confirmation/check-in/check-out readiness tests.
3. Add cancellation outcome/refund/forfeit review modeling.
4. Add workflow/staff-task mapping and approval-policy tests.
5. Only then add tool command drafts for collection/refund/waiver/forfeit execution boundaries.
