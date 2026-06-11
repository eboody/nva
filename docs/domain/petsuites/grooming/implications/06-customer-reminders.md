# Grooming operational implication 06: Customer reminders

Purpose: model the customer-reminder surface for PetSuites grooming without letting reminder timing, customer-contact permissions, appointment state, AI drafts, or provider message delivery collapse into generic follow-up helpers. This is a documentation/spec artifact for later Rust/domain cards; no live messages, bookings, payments, or provider changes are implied here.

Source context:

- `docs/domain/petsuites/grooming/service-domain-map.md`
- Existing `domain/src/operations.rs` grooming contract: `ReminderRule::{OneWeekBefore, FortyEightHoursBefore, MorningOf}`, `RebookingCadence`, `HistoryRequirement`, `NoShowPolicy`, `Contract::standard_petsuites()`.
- Existing cross-module surfaces: `operations::CustomerFollowUp`, `operations::FollowUpReason`, `operations::OperationsAction::DraftCustomerMessage`, `operations::StaffTaskKind::CustomerFollowUp`, `policy::ReviewGate::CustomerMessageApproval`, `agents::baseline_agent_specs()` including `grooming-rebooking`.

Assumptions:

- Customer identity, contact channels, consent, opt-out state, and household preferences are owned by `customer`/messaging policy, not grooming.
- Grooming may decide that a reminder is operationally due, draft content, and request a staff/customer-message review gate. It must not decide that a customer may legally be contacted without typed consent/context from the customer or messaging boundary.
- Provider delivery details (Gingr/SMS/email IDs, webhook event IDs, send receipts) stay in workflow/storage adapters and are promoted into semantic domain values before grooming behavior sees them.
- The safest extensible default is draft-first: agents may prepare reminders and staff tasks; member-facing send execution requires consent plus explicit approval or a deterministic pre-approved automation policy.

## 1. Operational story

### Trigger

A grooming reminder workflow begins from one of these domain events or scheduled scans:

1. A grooming appointment is scheduled, rescheduled, or placed on an approved hold.
2. The location reminder schedule reaches a configured timing rule, currently represented by `operations::grooming::ReminderRule::{OneWeekBefore, FortyEightHoursBefore, MorningOf}`.
3. A completed grooming service produces a rebooking recommendation, commonly the standard six-week PetSuites cadence or a groomer-recommended cadence.
4. A customer has not confirmed, prep information is missing, or a staff/groomer review created a customer follow-up task.
5. A provider webhook reports message delivery failure, opt-out, bounce, reply, appointment cancellation, or appointment time change.

### Actors

- Customer/member: receives only approved, consent-compatible reminders or follow-up messages.
- Front desk/staff: reviews reminder drafts, confirms customer preferences, resolves missing prep inputs, and may approve routine sends.
- Groomer: supplies prep instructions, style-specific warnings, product instructions, and review notes for messages that could affect grooming outcome.
- Manager: approves escalated or risky reminders, including complaint-sensitive, refund/deposit, repeated no-show, opt-out ambiguity, or policy-exception cases.
- Deterministic grooming policies: decide due timing, required review gates, and allowed action shape.
- AI grooming/rebooking agent: may rank opportunities and draft reminder text with evidence; it does not send, book, charge, waive, or alter provider state.
- Messaging/provider adapter: executes approved sends and returns typed delivery/audit results.

### Inputs

- `operations::grooming::Contract` for the location, especially reminder rules, rebooking cadence, no-show policy, and history requirement.
- Grooming appointment facts: appointment ID, location, customer, pet, offering/service plan, scheduled window, groomer assignment, reservation/add-on link, state, and last modification time.
- Customer contact facts: preferred channel, consent/opt-out status, channel availability, locale/timezone, household preferences, quiet hours, and prior delivery failures.
- Pet/grooming facts: service history, style notes, prep instructions, product instructions, handling/care review flags, and next-cadence recommendation.
- Workflow/audit facts: prior reminder attempts, approvals, provider delivery receipts, staff task status, and customer replies.

### Decisions

- Is the appointment/reminder target still valid and in a sendable state?
- Which semantic reminder kind is due: confirmation, prep instructions, forty-eight-hour reminder, morning-of reminder, rebooking due, lapsed-cadence winback, missing-information request, or no-show/deposit-sensitive follow-up?
- Does customer consent and channel policy allow a member-facing draft to become an approved send candidate?
- Does content contain sensitive care/medical/behavior/style/payment/refund/deposit/policy-exception material requiring groomer, manager, or customer-message approval?
- Has an equivalent reminder already been sent or acknowledged, making a new send duplicate/noisy?
- Should the workflow produce a message draft, an internal staff task, a manager escalation, or a disallowed decision?

### Outputs

- `operations::grooming::reminder::Plan`: a sequence of due reminder actions for a grooming appointment or rebooking recommendation.
- `operations::grooming::reminder::Draft`: typed customer-message draft with kind, audience, channel intent, content source, evidence, review gate, and idempotency key.
- `operations::StaffTask` or future `operations::grooming::reminder::StaffTaskRequest` for missing consent, missing prep information, failed delivery, reply review, or manager approval.
- `operations::OperationsAction::DraftCustomerMessage` or a more specific grooming reminder action once the domain surface is refined.
- `operations::grooming::reminder::AuditEvent` for due evaluation, draft creation, approval, send request, delivery result, cancellation/suppression, and reply handling.

### Success state

A successful reminder cycle leaves a truthful audit trail showing that:

- The reminder target was an active grooming appointment or rebooking recommendation at evaluation time.
- Timing and idempotency rules selected exactly the due reminder kind(s).
- Customer contact consent and approval gates were checked before any member-facing send.
- Either an approved reminder was delivered/queued through the messaging boundary, or a staff task/escalation was created with a clear reason.
- Appointment, reservation, payment, and customer identity state were not silently mutated by the reminder workflow.

### Failure and exception states

- `AppointmentNoLongerSendable`: appointment was cancelled, completed, no-showed, rescheduled, or missing required appointment identity.
- `ReminderAlreadySatisfied`: equivalent reminder was already sent, acknowledged, suppressed, or replaced by a newer appointment version.
- `ConsentUnavailableOrDenied`: no eligible channel, opt-out, expired consent, missing preference, or quiet-hours restriction.
- `CustomerMessageApprovalRequired`: draft is safe to prepare but not safe to send without review.
- `GroomerReviewRequired`: style/prep/product wording could affect grooming outcome and needs groomer confirmation.
- `ManagerApprovalRequired`: no-show/deposit, complaint, refund/discount, incident, policy exception, or repeated failed-contact context.
- `CareOrMedicalBoundary`: content references allergy, medical product, medication, injury, behavior, or handling facts that must route through care/medical review or use references rather than copied sensitive text.
- `DeliveryFailed`: provider bounce, SMS failure, email rejection, webhook timeout, or missing provider receipt.
- `ProviderPayloadUntrusted`: raw provider IDs/status strings cannot be promoted into semantic reminder state.

## 2. Domain types to add or refine

Use semantic paths under `operations::grooming::reminder` when the concept is specifically about grooming reminders. Reuse parent modules where they own identity, contact, or policy.

### New/refined entities

- `operations::grooming::AppointmentId`
  - Non-empty provider/domain grooming appointment identifier. Provider IDs are parsed at the boundary through named constructors.
- `operations::grooming::AppointmentVersion`
  - Monotonic provider version, updated-at timestamp, or explicit revision token used for reminder idempotency after reschedules.
- `operations::grooming::reminder::Plan`
  - Aggregate for the reminder decisions for one appointment/rebooking target. Invariant: non-empty only when at least one action is due or one exception/staff task must be recorded.
- `operations::grooming::reminder::Draft`
  - Member-facing message draft. Invariant: cannot be constructed without customer ID, pet ID, location ID, reminder kind, content, source/evidence, review decision, and idempotency key.
- `operations::grooming::reminder::Attempt`
  - Durable attempt record for draft approval/send/delivery. Invariant: send attempts reference an approved draft and an external delivery request/receipt if available.

### Value objects and scalar contracts

- `operations::grooming::reminder::Kind`
  - `AppointmentConfirmation`, `PrepInstructions`, `OneWeekBefore`, `FortyEightHoursBefore`, `MorningOf`, `RebookingDue`, `LapsedCadenceWinback`, `MissingPrepInformation`, `DeliveryFailureFollowUp`, `CustomerReplyReview`.
  - This refines timing-only `ReminderRule` into the reason/content intent. Keep `ReminderRule` as location timing policy until migrated.
- `operations::grooming::reminder::TimingRule`
  - A semantic wrapper around `ReminderRule` plus channel/quiet-hours behavior if later needed. Invariant: due-at calculations use location/customer timezone rather than naked UTC math at call sites.
- `operations::grooming::reminder::DueAt`
  - Validated timestamp for the reminder action; derived by policy, not arbitrary caller input.
- `operations::grooming::reminder::IdempotencyKey`
  - Stable key across retries, likely `{appointment_id}:{appointment_version}:{kind}:{channel}` or `{recommendation_id}:{kind}:{channel}`. Invariant: no duplicate send for the same semantic reminder target.
- `operations::grooming::reminder::Content`
  - Trimmed, bounded message body. Separate customer-safe content from internal evidence/notes. Redacted debug if content can include customer/pet specifics.
- `operations::grooming::reminder::TemplateName`
  - Validated local/provider template identifier; not a raw arbitrary string in the domain core.
- `operations::grooming::reminder::Evidence`
  - References appointment, contract rule, service history, rebooking recommendation, customer consent snapshot, and prior attempt IDs used to justify the plan.
- `operations::grooming::reminder::SuppressionReason`
  - `NoConsent`, `OptedOut`, `QuietHours`, `AlreadyAcknowledged`, `AppointmentCancelled`, `AppointmentRescheduled`, `ManagerHold`, `Duplicate`, `UnsafeContent`, `ProviderUnavailable`.
- `operations::grooming::reminder::ApprovalDecision`
  - `DraftOnly`, `CustomerMessageApprovalRequired`, `GroomerApprovalRequired`, `ManagerApprovalRequired`, `ApprovedForSend(policy::ApprovalToken)`, `Disallowed(policy::PolicyDenialReason)`.
  - Do not model this as a boolean.
- `operations::grooming::reminder::DeliveryState`
  - `NotRequested`, `Queued`, `Sent`, `Delivered`, `Failed`, `Bounced`, `Suppressed`, `Acknowledged`, `ReplyNeedsReview`.
- `operations::grooming::reminder::ChannelIntent`
  - Desired channel chosen by policy from customer preferences, e.g. `Preferred`, `Email`, `Sms`, `Portal`, `PhoneTask`. Actual provider delivery channel remains boundary-owned.

### Policies, repositories, and ports

- `operations::grooming::reminder::Policy`
  - Decides due reminders, suppression, approval gates, and staff task needs.
- `operations::grooming::reminder::ContentPolicy`
  - Checks whether a draft stays customer-safe and whether content needs groomer/manager/care review.
- `operations::grooming::reminder::Repository`
  - Loads and writes reminder plans, drafts, attempts, and audit events in domain form.
- `operations::grooming::reminder::DeliveryPort`
  - Boundary port accepting only approved send commands and returning typed delivery outcomes. This is not a free helper or provider client hidden in the domain.
- `operations::grooming::reminder::AuditTrail`
  - Append-only event store/repository boundary for approval and delivery traceability.

## 3. Relationship map between types

### Entities and aggregate roots

- `operations::grooming::Appointment` or current appointment snapshot owns schedule state and appointment version. Reminder policy reads it but does not mutate schedule state.
- `operations::grooming::RebookingRecommendation` can be a reminder target for due/overdue rebooking messages.
- `operations::grooming::reminder::Plan` is the reminder aggregate for one appointment/rebooking target.
- `operations::grooming::reminder::Draft` is a child entity of a plan and becomes sendable only through approval.
- `operations::grooming::reminder::Attempt` records a delivery attempt for one approved draft.

### Value objects

- `reminder::Kind`, `TimingRule`, `DueAt`, `IdempotencyKey`, `ChannelIntent`, `Content`, `TemplateName`, `Evidence`, `SuppressionReason`, `DeliveryState`, and `ApprovalDecision` make reminder semantics explicit.
- `customer::ContactPreference`, `customer::CommunicationConsent`, `customer::OptOutState`, or equivalent customer-owned values are inputs, not owned/refined by grooming.
- `operations::grooming::AppointmentMinutes`, `RebookingCadence`, and `CadenceWeeks` remain grooming-owned timing/cadence context but should not be overloaded as message rules.

### Policies

- `operations::grooming::reminder::Policy` owns due/suppression/approval decisions.
- `operations::grooming::reminder::ContentPolicy` owns safety/review classification of customer wording.
- `operations::grooming::RebookingPolicy` owns whether a pet/customer is due for a rebooking recommendation; reminder policy owns how/when to contact about it.
- `operations::grooming::AutomationPolicy` or parent `policy` owns final mapping from recommendation/draft to approval gate.
- `customer::CommunicationPolicy` or messaging policy owns contact permission, preferred channel, quiet hours, and opt-out interpretation.

### Repositories/stores

- `operations::grooming::Repository`/`ContractRepository` loads grooming contract and appointment/rebooking snapshots.
- `operations::grooming::reminder::Repository` persists plans, drafts, attempts, idempotency keys, and audit events.
- `customer::Repository` or a customer contact store provides consent/preference snapshots in customer-owned types.
- `workflow::Repository` or provider-specific storage records raw delivery payloads and promotes them to `reminder::DeliveryState`/`Attempt`.

### Workflow events

- `workflow::WorkflowEvent::GroomingAppointmentScheduled` or a future equivalent triggers initial plan creation.
- `workflow::WorkflowEvent::ScheduledReminderDue` triggers due evaluation.
- `workflow::WorkflowEvent::GroomingAppointmentRescheduled` invalidates stale idempotency keys and drafts.
- `workflow::WorkflowEvent::CustomerMessageDeliveryUpdated` updates attempt/delivery state.
- `workflow::WorkflowEvent::CustomerReplyReceived` creates `CustomerReplyReview` or a staff task rather than treating reply parsing as automatic state change.

### Staff tasks

- Existing `operations::StaffTaskKind::CustomerFollowUp { reason }` may cover ordinary human follow-up.
- Add grooming-specific task kinds only when semantics require them, for example:
  - `GroomingReminderReview { appointment_id, draft_id }`
  - `GroomingPrepInformationReview { appointment_id, pet_id }`
  - `GroomingRebookingFollowUp { recommendation_id }`
  - `GroomingDeliveryFailureFollowUp { draft_id }`
- Use `StaffTaskAssignment::Role(StaffRole::Groomer)` or manager role when review ownership matters.

### Agent specs/tools

- Existing `agents::baseline_agent_specs()` includes `grooming-rebooking`, allowed tools `grooming-history-read`, `availability-read`, `draft-message`, and forbidden actions `book grooming slot`, `apply discount`, `send message without approval`.
- Add a future `grooming-reminder-drafter` spec only if reminder drafting needs a separate agent contract from rebooking. Its output schema should be `operations::grooming::reminder::DraftRecommendation`, not raw text.
- Tools may read appointment/history/availability and create draft messages or internal tasks. Tools may not send customer messages unless the input command carries typed consent, approval, and idempotency evidence.

## 4. Interaction contract

Rust-like pseudo-signatures show ownership and boundaries. Names can change during implementation, but behavior should stay on truthful owners.

```rust
pub mod operations::grooming::reminder {
    pub struct Policy;

    impl Policy {
        pub fn plan_for_appointment(
            &self,
            contract: &operations::grooming::Contract,
            appointment: &operations::grooming::AppointmentSnapshot,
            customer_contact: &customer::communication::Snapshot,
            prior_attempts: &[Attempt],
            now: time::Now,
        ) -> Result<Plan>;

        pub fn plan_for_rebooking(
            &self,
            contract: &operations::grooming::Contract,
            recommendation: &operations::grooming::RebookingRecommendation,
            customer_contact: &customer::communication::Snapshot,
            prior_attempts: &[Attempt],
            now: time::Now,
        ) -> Result<Plan>;

        pub fn approveability_for(
            &self,
            draft: &Draft,
            customer_contact: &customer::communication::Snapshot,
            context: &ApprovalContext,
        ) -> ApprovalDecision;
    }

    pub struct ContentPolicy;

    impl ContentPolicy {
        pub fn classify(
            &self,
            draft: DraftCandidate,
            care_refs: &[care::ReviewFlagRef],
            payment_context: Option<payment::DepositRequirement>,
        ) -> ContentDecision;
    }
}
```

Behavior notes:

- `ReminderPolicy::plan_for_appointment` owns timing, suppression, duplicate detection, and reminder kind selection.
- `ContentPolicy::classify` owns customer-message safety. It does not load appointments or send messages.
- Customer communication policy owns consent/channel eligibility. Grooming receives a typed snapshot rather than inspecting raw email/SMS booleans.
- `Draft::approve_with(token)` may produce an `ApprovedDraft`; without a token the delivery port should not accept the draft.
- Delivery is a port on the reminder boundary, not a general helper function:

```rust
pub trait Repository {
    fn load_plan(&self, target: ReminderTarget) -> Result<Option<Plan>>;
    fn save_plan(&self, plan: &Plan) -> Result<()>;
    fn save_draft(&self, draft: &Draft) -> Result<()>;
    fn record_attempt(&self, attempt: &Attempt) -> Result<()>;
    fn append_audit(&self, event: AuditEvent) -> Result<()>;
}

pub trait DeliveryPort {
    fn enqueue_approved_send(
        &self,
        command: ApprovedSendCommand,
    ) -> Result<DeliveryRequestReceipt>;
}

pub struct ApprovedSendCommand {
    pub draft: ApprovedDraft,
    pub consent_snapshot: customer::communication::ConsentSnapshot,
    pub idempotency_key: IdempotencyKey,
    pub audit_context: policy::ApprovalAuditContext,
}
```

- `DeliveryPort` cannot accept `Draft` directly. It must accept `ApprovedDraft` or an equivalent typestate/enum variant proving approval and consent.
- Provider callbacks are promoted through named constructors:

```rust
impl Attempt {
    pub fn from_provider_receipt(
        draft_id: DraftId,
        receipt: messaging::ProviderReceipt,
    ) -> Result<Self>;
}
```

- Existing coarse `OperationsAction::DraftCustomerMessage { customer_id, reason }` can be used as an integration bridge, but later code should prefer a grooming-specific action carrying `appointment_id`, `pet_id`, `reminder::Kind`, `idempotency_key`, and `ApprovalDecision`.

## 5. Review and approval contract

### Automation level

- Due detection: safe to automate when inputs are typed and read-only.
- Draft creation: safe as draft-only when content is grounded in appointment/contract/history evidence.
- Internal task creation: safe when the task clearly labels missing approval/consent/review reason.
- Member-facing send: requires customer consent plus `CustomerMessageApproval` unless a deterministic policy defines a narrow pre-approved send class.
- Provider state changes: reminders may not book, reschedule, cancel, charge, waive, discount, or modify reservations/invoices.

### Review gates

- `policy::ReviewGate::CustomerMessageApproval`
  - Any member-facing reminder, confirmation, rebooking offer, prep instructions, or winback message.
- Groomer/staff review
  - Style instructions, product/prep language, first-time service ambiguity, low-confidence service interpretation, missing pet coat/breed/profile facts.
- Manager approval
  - Deposit/no-show/late cancellation wording, discounts/offers/waivers, complaint-sensitive context, repeated delivery failures, escalated customer replies, safety/legal/incident context.
- Care/medical/behavior review
  - Medical shampoo, allergy/skin condition, injury, medication, handling/behavior flags, or anything that could be interpreted as medical advice.

### Audit trail

The reminder audit trail should capture:

- Trigger event and workflow run ID.
- Appointment/rebooking target IDs and appointment version.
- Contract snapshot/version and reminder rule used.
- Customer communication consent snapshot and channel decision.
- Draft content/template identity, evidence references, and AI model/spec if generated.
- Review gate, approver identity/token, timestamp, and decision rationale.
- Idempotency key and duplicate/suppression checks.
- Provider delivery request/receipt and callback state.
- Customer reply routing and staff task IDs.

### Customer/member-facing boundaries

Never send or imply approval for:

- A message to a customer who has opted out or lacks eligible channel consent.
- Deposit, fee, refund, discount, or waiver statements without payment/manager approval.
- Medical suitability or diagnosis for a product/service.
- A booking/reschedule/cancellation confirmation unless the provider appointment state is already true and the confirmation is approved.
- Style/prep instructions generated from ambiguous groomer notes without groomer/staff review.
- Any apology, incident, injury, complaint, or reputation-sensitive wording without manager review.

## 6. Test contracts

Future implementation should add semantic tests named after domain truths. Suggested tests:

1. `grooming_reminder_policy_creates_forty_eight_hour_and_morning_of_plan_from_standard_contract`
   - `Contract::standard_petsuites()` produces due reminder actions for the existing two timing rules when appointment/customer inputs are valid.
2. `grooming_reminder_policy_does_not_send_when_customer_consent_is_missing`
   - Plan may include draft/staff task, but `ApprovalDecision` is not approved for send.
3. `grooming_reminder_idempotency_key_prevents_duplicate_member_facing_send_for_same_appointment_version_kind_and_channel`
   - Re-running the due scan does not create a second send command for the same semantic target.
4. `grooming_appointment_reschedule_invalidates_stale_reminder_draft_without_reusing_old_send_approval`
   - Appointment version changes force a new plan/draft and preserve an audit event for the stale draft.
5. `grooming_morning_of_reminder_is_suppressed_after_appointment_cancelled_or_completed`
   - No reminder send command is produced for non-sendable appointment states.
6. `grooming_rebooking_due_reminder_uses_rebooking_recommendation_without_mutating_calendar_or_reservation`
   - Rebooking reminder output is a draft/customer follow-up, not an appointment booking or reservation mutation.
7. `grooming_prep_instruction_content_with_medical_product_requires_care_or_manager_review`
   - Medical/sensitive product wording cannot be approved by routine customer-message gate alone.
8. `grooming_style_note_reminder_requires_groomer_review_before_customer_message_approval`
   - Style interpretation has a groomer/staff review gate before member-facing approval.
9. `grooming_deposit_or_no_show_follow_up_requires_manager_approval`
   - No-show/deposit reminders cannot be treated as ordinary appointment reminders.
10. `grooming_delivery_failure_creates_staff_follow_up_task_with_provider_receipt_evidence`
    - Failed provider callback becomes a typed attempt plus staff task.
11. `grooming_customer_reply_routes_to_review_instead_of_changing_appointment_state_automatically`
    - Reply text is not parsed into booking/cancellation state without staff/provider confirmation.
12. `grooming_reminder_delivery_port_rejects_unapproved_draft`
    - Compile-time or runtime contract prevents sending an unapproved `Draft`.
13. `grooming_reminder_audit_trail_records_trigger_consent_review_idempotency_and_delivery_receipt`
    - Required audit facts exist for approved and suppressed paths.
14. `grooming_reminder_storage_promotes_provider_status_strings_to_delivery_state_or_rejects_unknown_values`
    - Boundary storage rejects unknown/untrusted provider states instead of leaking strings into domain logic.
15. `grooming_rebooking_agent_output_must_include_evidence_review_boundary_and_forbid_send_without_approval`
    - Agent output validates into typed recommendations only when evidence and approval boundary are explicit.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add or split `operations::grooming::reminder` module; refine `ReminderRule` into timing vs kind/content policy; add reminder plan/draft/attempt/audit types.
- `domain/src/customer.rs` or a new `domain/src/customer/communication.rs`
  - Add customer-owned consent/preference/channel snapshot types if not already present.
- `domain/src/policy.rs`
  - Reuse `ReviewGate::CustomerMessageApproval`; consider explicit approval token/audit context and richer denial/review reasons for grooming reminders.
- `domain/src/workflow.rs`
  - Add workflow events for appointment scheduled/rescheduled, reminder due, delivery updated, customer reply received, and draft approved if the workflow layer does not already expose them.
- `domain/src/agents.rs`
  - Either refine `grooming-rebooking` output schema or add `grooming-reminder-drafter` with forbidden actions matching the approval contract.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Extend current grooming contract tests for standard reminder rules.
- `domain/tests/domain_quality_patterns.rs` or a new `domain/tests/grooming_reminder_contracts.rs`
  - Add semantic policy, approval, idempotency, and delivery-port tests.
- `storage` crate tests/code, if present in the serialized card
  - Add records/codecs for reminder draft/attempt/audit state, provider delivery receipt promotion, and unknown provider status rejection.

### Migration/refactor risks

- Existing `ReminderRule` currently mixes timing policy with reminder purpose. Avoid breaking storage compatibility by introducing `reminder::TimingRule`/`Kind` with conversion rather than renaming in place without migration.
- Existing `OperationsAction::DraftCustomerMessage` and `FollowUpReason` are too coarse for grooming reminder idempotency. They can bridge early integration but should not become the long-term source of truth.
- Customer contact consent may not yet exist as a rich domain type. Do not place consent booleans inside grooming to move faster; create customer-owned communication snapshots or adapters.
- Appointment state/version may not yet be modeled. Reminder idempotency needs some version/revision concept to avoid stale sends after reschedules.
- Provider messaging statuses are likely stringly typed. Boundary code must map known statuses to `DeliveryState` or reject/park unknown statuses for review.
- AI-generated reminder content can accidentally copy sensitive groomer/care notes. Keep evidence/reference fields separate from customer-visible content and use redacted debug for content-bearing types.
- Typestate (`Draft` -> `ApprovedDraft`) may require more scaffolding than an enum. If the first card uses an enum, preserve the invariant that delivery APIs reject unapproved variants.

### Dependencies on other implications

- Rebooking cadence/service history implication: reminder due and lapsed-cadence winback depend on `RebookingRecommendation` and `ServiceHistoryEntry` semantics.
- Groomer notes/history implication: prep/style reminders depend on separating style notes from care/medical/handling references.
- No-show/cancellation implication: deposit or repeated no-show follow-ups require manager/payment-policy review.
- Calendar/scheduling implication: appointment reminders depend on a stable appointment ID, schedule window, state, and version.
- Cross-sell implication: exit-bath or grooming offer messages must remain recommendations/drafts and not mutate reservations or invoices.
- Customer/contact implication outside grooming: consent, opt-out, preferred channel, quiet hours, and delivery preferences should be modeled outside the grooming bounded context.

Implementation entry rule: start with failing semantic tests for idempotency, consent gating, and unapproved-draft delivery rejection; then add the smallest `operations::grooming::reminder` types/policies needed. Do not implement reminders as `utils::send_reminder(customer_id, text)` or as a bare `bool approved` flag.
