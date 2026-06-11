# Training operational implication 07: Packages and recurring engagement

Purpose: model how PetSuites training packages, unused sessions, completion follow-up, and re-enrollment opportunities become safe operational work. This is a domain contract for later Rust implementation, not an automation launch plan.

Assumption: package and recurring-engagement details vary by resort and may be stored in Gingr or another operating system. The safest extensible model is to treat provider data as boundary evidence, promote it into typed training package state, and allow automation to recommend staff-reviewed tasks/messages rather than directly modifying packages or contacting customers.

## 1. Operational story

### Trigger

The package/recurring-engagement workflow starts when one of these events is observed:

- A customer asks about buying a training package, board-and-train bundle, group class sequence, private lesson package, or recurring training cadence.
- A trainer or front-desk staff member enrolls a pet into a package-backed program.
- A training session is completed and package usage must be reconciled.
- A program is completed and follow-up/re-enrollment should be considered.
- A package has unused sessions, expiring eligibility, missing payment authorization, or a mismatch between scheduled sessions and purchased units.
- A daily/weekly manager brief detects lapsed training engagement or safe next-step opportunities from completed outcomes.

### Actors

- Customer/parent: purchases or asks about packages, receives approved follow-up, and chooses whether to re-enroll.
- Pet: the subject of training outcomes, eligibility, and package usage.
- Trainer: documents session completion, progress, outcomes, homework, package fit, and recommended next step.
- Front desk / client-service staff: explains approved package options, schedules sessions, reconciles package state, and sends approved messages.
- Manager: approves overrides, credits/refunds/comps, package-rule changes, and sensitive escalations.
- AI agent: drafts recommendations, identifies gaps, summarizes evidence, and creates internal review tasks; it does not approve customer-facing claims, capture payment, adjust package balances, or mutate bookings.
- Provider/storage systems: source of reservation, package ledger, payment, schedule, and communication records.

### Inputs

- `operations::training::Enrollment` and selected `operations::training::Program` / `Offering`.
- Current `operations::training::PackagePolicy` and location-specific `RecurringEngagementPolicy`.
- Package ledger facts: purchased sessions, consumed sessions, reserved future sessions, expiration/validity window, package status, and provider references.
- Session/progress evidence: completed `ProgressReport`, milestone states, documented `OutcomeDocumentation`, missed/no-show/cancelled sessions, and trainer notes.
- Customer facts: customer ID, communication preference, prior opt-out/suppression state, package purchase history, and reservation context.
- Pet facts: pet ID, care/temperament review state, behavior goals, and safety-sensitive restrictions.
- Payment facts: deposit/payment authorization status, outstanding balance, refund/credit status, and comp/waiver indicators.
- Location facts: active offerings, trainer capacity, trainer qualifications, group class calendars, and local package rules.

### Decisions

- Is the requested offering package-backed, pay-per-session, a board-and-train bundle, or intentionally not sold as a package?
- Is the package eligible for the selected pet, program, location, reservation context, and trainer capacity?
- Does scheduling or completing this session consume a package unit, reserve a future unit, or require human reconciliation?
- Is there a payment/package blocker that prevents scheduling, completion, or follow-up?
- Should unused sessions produce an internal follow-up task, a rebooking prompt, or no action?
- Does program completion justify a next-step recommendation such as CGC prep, private lessons, group class continuation, refresher sessions, or no upsell?
- Is the recommendation safe and useful, or is it blocked by care/behavior/safety/payment/customer-preference facts?
- What review gate is required before a message, booking change, credit/refund, package modification, or outcome claim can leave the system?

### Outputs

- A typed `PackageLedgerDecision` describing whether usage is `Reserve`, `Consume`, `ReleaseReservation`, `NeedsReconciliation`, or `Blocked`.
- A typed `RecurringEngagementDecision` describing `NoAction`, `CreateStaffTask`, `DraftParentMessage`, `RecommendReEnrollment`, `RouteToTrainerReview`, or `RouteToManagerReview`.
- Staff tasks for package reconciliation, overdue follow-up, unused-session outreach, lapsed-engagement review, or next-program consultation.
- Workflow drafts for approved-review queues: message draft, booking-change draft, payment/package-review draft, or manager-escalation draft.
- Audit events that connect package decisions to enrollment, package ledger snapshot, trainer evidence, policy version, actor, review gate, and final disposition.

### Success state

A package-backed training engagement is successful when:

- purchased, reserved, consumed, expired, credited, and refunded sessions are represented as typed ledger entries rather than raw counters;
- every schedule/completion operation has an explicit package decision;
- unused sessions and appropriate next steps produce staff-visible follow-up work;
- no member-facing message, payment adjustment, package modification, or outcome claim occurs without the required human approval;
- the audit trail can explain why an opportunity was recommended, suppressed, blocked, or escalated.

### Failure and exception states

- `PackageMismatch`: scheduled/completed session count exceeds available package units, or provider ledger disagrees with domain snapshot.
- `MissingPackageLedger`: package-backed program has no ledger snapshot or provider reference.
- `PaymentRequired`: package or session cannot proceed until payment/deposit authorization is resolved.
- `ExpiredOrIneligiblePackage`: package is expired, location-ineligible, pet-ineligible, program-ineligible, or not valid for the requested trainer/class.
- `TrainerCapacityUnavailable`: re-enrollment recommendation exists, but capacity prevents a booking prompt.
- `BehaviorOrCareReviewRequired`: pet facts make automated upsell/follow-up unsafe until trainer/manager review.
- `CustomerCommunicationSuppressed`: opt-out, complaint state, or recent negative interaction suppresses member-facing drafts.
- `ManagerApprovalRequired`: credit/refund/comp/override/package extension is needed.
- `ProviderWriteRejected`: provider tool rejects a staff-approved change; domain records a failed draft/write result rather than pretending the package changed.

## 2. Domain types to add or refine

### Package and ledger identity

- `operations::training::package::Id`
  - Stable package ID distinct from enrollment, reservation, payment, and provider transaction IDs.
  - Invariant: non-empty/UUID-backed; parsed at storage/provider boundary.
- `operations::training::package::ProviderRef`
  - Boundary reference for Gingr or another system.
  - Invariant: never used as the domain package ID; may be absent for manual/imported packages.
- `operations::training::package::PolicyVersion`
  - Identifies the package-rule snapshot used for a decision.
  - Invariant: every package/recurring decision records the policy version used.

### Package shape and eligibility

Refine current `operations::training::PackagePolicy` into a more expressive truthful shape:

```rust
pub enum operations::training::PackagePolicy {
    PayPerSession,
    MultiSessionPackage { sessions: training::SessionCount },
    BoardAndTrainBundle,
    ClassSeries { sessions: training::SessionCount },
    RecurringCadence { cadence: training::engagement::Cadence },
    NotPackageEligible { reason: training::package::IneligibilityReason },
}
```

Add:

- `operations::training::package::Eligibility`
  - `Eligible`, `Ineligible { reasons: Vec<IneligibilityReason> }`, `NeedsReview { gates: Vec<TrainingReviewGate> }`.
  - Invariant: scheduling against a package requires `Eligible` or an explicit staff-approved review outcome.
- `operations::training::package::IneligibilityReason`
  - `WrongLocation`, `WrongProgramKind`, `WrongPet`, `Expired`, `NoRemainingSessions`, `PaymentRequired`, `TrainerRequirementMismatch`, `ClassSeriesMismatch`, `ReservationContextRequired`, `BehaviorOrCareReviewRequired`, `CustomerSuppressed`.
- `operations::training::package::ValidityWindow`
  - Date/time window where a package can be used.
  - Invariant: start <= end; open-ended packages require explicit `NoExpiration` variant rather than a sentinel date.

### Ledger and usage state

- `operations::training::package::Ledger`
  - Entity owned by the training package aggregate.
  - Fields should include package ID, customer ID, pet ID when pet-specific, location ID, policy, status, purchased/reserved/consumed counts, validity, provider ref, and audit cursor.
  - Invariant: consumed + reserved cannot exceed purchased unless a manager-approved overrun/credit entry exists.
- `operations::training::package::Status`
  - `Quoted`, `PendingPayment`, `Active`, `Exhausted`, `Expired`, `Suspended`, `Cancelled`, `Credited`, `Refunded`, `TransferredWithManagerApproval`.
- `operations::training::package::SessionBalance`
  - Value object over `purchased`, `reserved`, `consumed`, `remaining_available`.
  - Invariant: non-negative counts; calculation is method-owned, not caller-computed.
- `operations::training::package::LedgerEntry`
  - `Purchased`, `ReservedForSession`, `ConsumedByCompletedSession`, `ReservationReleased`, `Expired`, `Credited`, `Refunded`, `AdjustedByManager`, `ImportedFromProvider`.
  - Invariant: entries carry typed actor/review/audit metadata; manager-only entries require `ManagerApproval`.
- `operations::training::package::UsageDecision`
  - `Reserve { package_id, session_ref }`, `Consume { package_id, session_ref }`, `ReleaseReservation { package_id, session_ref }`, `NoPackageUsage`, `Blocked { blockers }`, `NeedsReconciliation { reasons }`.

### Recurring engagement and upsell

- `operations::training::engagement::Cadence`
  - `AfterEachSession`, `AfterProgramCompletion`, `ThirtyDaysAfterCompletion`, `EveryWeeks(CadenceWeeks)`, `BeforePackageExpiration`, `WhenUnusedSessionsRemain`.
  - Refines current `FollowUpCadence` by separating staff follow-up timing from package shape.
- `operations::training::engagement::Candidate`
  - Evidence-backed candidate for continued training.
  - Invariant: candidate references enrollment/package/progress/outcome evidence; it cannot be constructed from AI prose alone.
- `operations::training::engagement::Recommendation`
  - `NoAction`, `UseRemainingSessions`, `SchedulePrivateLesson`, `EnrollInGroupClass`, `ContinueClassSeries`, `CgcPrep`, `RefresherPackage`, `TrainerConsult`, `ManagerReview`, `Suppress`.
- `operations::training::engagement::SuppressReason`
  - `NoEvidence`, `RecentComplaint`, `CustomerOptedOut`, `BehaviorSafetySensitive`, `TrainerCapacityUnavailable`, `PaymentIssue`, `AlreadyScheduled`, `PackageExhausted`, `ManagerHold`.
- `operations::training::UpsellOpportunity`
  - Should carry `customer_id`, `pet_id`, `location_id`, recommendation, evidence, approval boundary, and allowed next action.
  - Invariant: opportunity is not a send permission.

### Review, audit, and safe text

- `operations::training::package::AuditEvent`
  - Records package decisions and ledger mutations with actor, source event, policy version, before/after balance, review gate, and provider-write result.
- `operations::training::ParentFollowUp`
  - Refine into internal task/draft aggregate with `approval_boundary`, `reason`, `evidence`, `draft_message`, and `send_state`.
- `operations::training::package::ReconciliationReason`
  - `ProviderBalanceDisagrees`, `SessionCompletedWithoutReservation`, `ReservationWithoutCompletion`, `PaymentStateChanged`, `RefundOrCreditPending`, `ImportedPackageAmbiguous`.

## 3. Relationship map

### Entities

- `operations::training::Enrollment` references package state; it does not own the whole ledger.
- `operations::training::package::Ledger` owns purchased/reserved/consumed/adjusted package facts.
- `operations::training::Session` or later session aggregate is the event source for reservation/consumption decisions.
- `operations::training::ProgressReport` and `OutcomeDocumentation` are evidence sources for recurring engagement.
- `operations::StaffTask` represents internal package-review/follow-up work.
- `workflow::DraftMessage`, `workflow::DraftTask`, and `workflow::RecommendedAction` carry agent/staff action drafts.

### Value objects

- `package::Id`, `package::ProviderRef`, `package::PolicyVersion`, `package::ValidityWindow`, `package::SessionBalance`, `package::LedgerEntry`, `engagement::Cadence`, `engagement::EvidenceSummary`, `training::SessionCount`, `training::SessionMinutes`, `money::Money`, and `payment::DepositStatus`.

### Policies

- `training::package::EligibilityPolicy` decides whether a package can be used for a selected enrollment/session/program.
- `training::package::LedgerPolicy` decides reserve/consume/release/reconcile outcomes.
- `training::engagement::RecurringEngagementPolicy` decides no action, task, draft, trainer review, or manager review.
- `training::engagement::SuppressionPolicy` prevents unsafe, annoying, or noncompliant outreach.
- `training::ProgressReportPolicy` and `training::OutcomeDocumentationPolicy` determine whether evidence is sufficient for customer-facing claims.
- `payment` policies own authorization/capture/refund truth; training consumes typed payment status only.

### Repositories and stores

- `operations::training::package::Repository` loads/saves package ledger snapshots and entries.
- `operations::training::package::ProviderReconciliationStore` stores provider ledger snapshots and reconciliation findings.
- `operations::training::enrollment::Repository` loads enrollments and package refs.
- `operations::training::progress::Repository` loads progress/outcome evidence.
- `operations::training::offering::Repository` loads active offerings and package policies by location.
- `operations::training::trainer::ScheduleRepository` checks capacity before recommending recurrence.
- `workflow::Repository` or `workflow::DraftStore` persists review-gated tasks/messages.

### Workflow events

- `TrainingPackageQuoted`
- `TrainingPackagePurchased`
- `TrainingPackagePaymentPending`
- `TrainingSessionReservedPackageUnit`
- `TrainingSessionConsumedPackageUnit`
- `TrainingPackageUsageNeedsReconciliation`
- `TrainingProgramCompleted`
- `TrainingUnusedSessionsDetected`
- `TrainingRecurringEngagementRecommended`
- `TrainingParentFollowUpDrafted`
- `TrainingPackageAdjustedWithManagerApproval`

### Staff tasks

Add training-specific staff task semantics instead of generic title-only tasks:

```rust
operations::StaffTaskKind::TrainingPackageReview { enrollment_id, package_id }
operations::StaffTaskKind::TrainingPackageReconciliation { package_id, reason }
operations::StaffTaskKind::TrainingUnusedSessionFollowUp { package_id, customer_id, pet_id }
operations::StaffTaskKind::TrainingReEnrollmentConsult { enrollment_id, recommendation }
operations::StaffTaskKind::TrainingParentMessageApproval { enrollment_id, gate }
```

### Agent specs and tools

- `agents::training::package_monitor::Spec`: reads package ledgers/progress snapshots and emits recommendations/tasks only.
- `agents::training::recurring_engagement::Spec`: drafts safe staff-reviewed follow-up copy with evidence citations.
- `tools::training::package::DraftAdjustment`: provider mutation draft; requires manager approval before execution.
- `tools::training::booking::DraftScheduleChange`: booking/session draft; requires staff approval before execution.
- `tools::communication::DraftCustomerMessage`: member-facing draft; requires staff approval, and manager approval if safety/complaint/payment-sensitive.

## 4. Interaction contract

Use behavior owners that preserve domain meaning. Avoid free-floating helpers like `calculate_remaining_sessions()` or `should_upsell()`.

```rust
impl training::package::Ledger {
    pub fn balance(&self) -> package::SessionBalance;

    pub fn reserve_for_session(
        &self,
        session: &training::SessionRef,
        policy: &package::LedgerPolicy,
        ctx: &package::UsageContext,
    ) -> package::Result<package::UsageDecision>;

    pub fn consume_completed_session(
        &self,
        completed: &training::CompletedSession,
        policy: &package::LedgerPolicy,
        approval: package::UsageApproval,
    ) -> package::Result<package::LedgerEntry>;

    pub fn apply_manager_adjustment(
        &self,
        adjustment: package::ManagerAdjustment,
        approval: policy::ManagerApproval,
    ) -> package::Result<package::LedgerEntry>;
}
```

```rust
impl training::package::EligibilityPolicy {
    pub fn evaluate(
        &self,
        package: &package::Ledger,
        enrollment: &training::Enrollment,
        offering: &training::Offering,
        reservation: Option<&entities::Reservation>,
        payment: &payment::StatusSnapshot,
        care_review: &training::CareReviewSnapshot,
    ) -> package::Eligibility;
}
```

```rust
impl training::package::LedgerPolicy {
    pub fn decide_usage(
        &self,
        ledger: &package::Ledger,
        event: package::UsageEvent,
        eligibility: &package::Eligibility,
    ) -> package::UsageDecision;
}
```

```rust
impl training::engagement::RecurringEngagementPolicy {
    pub fn evaluate(
        &self,
        candidate: engagement::Candidate,
        package: Option<&package::Ledger>,
        outcomes: &[training::OutcomeDocumentation],
        progress: &[training::ProgressReport],
        customer: &training::CustomerEngagementSnapshot,
        capacity: &training::TrainerCapacitySnapshot,
        offerings: &training::OfferingCatalog,
    ) -> engagement::Decision;
}
```

```rust
impl training::FollowUpService {
    pub fn propose_package_follow_up(
        &self,
        decision: engagement::Decision,
        approval_policy: &training::ApprovalPolicy,
    ) -> workflow::Result<training::ParentFollowUp>;
}
```

```rust
pub trait training::package::Repository {
    fn get(&self, id: package::Id) -> package::Result<package::Ledger>;
    fn find_active_for_customer_pet(
        &self,
        customer_id: entities::CustomerId,
        pet_id: entities::PetId,
        location_id: entities::LocationId,
    ) -> package::Result<Vec<package::Ledger>>;
    fn append_entry(&self, id: package::Id, entry: package::LedgerEntry) -> package::Result<package::Ledger>;
    fn save_reconciliation(
        &self,
        finding: package::ReconciliationFinding,
    ) -> package::Result<()>;
}
```

```rust
pub trait training::engagement::WorkflowSink {
    fn create_staff_task(&self, task: operations::StaffTask) -> workflow::Result<workflow::TaskId>;
    fn save_parent_message_draft(&self, draft: workflow::DraftMessage) -> workflow::Result<workflow::DraftId>;
    fn record_recommendation(&self, action: workflow::RecommendedAction) -> workflow::Result<workflow::ActionId>;
}
```

Behavior ownership notes:

- `package::Ledger` owns balance math and entry invariants.
- `package::EligibilityPolicy` owns package-use eligibility, not booking tools or payment capture.
- `package::LedgerPolicy` owns reserve/consume/reconcile decisions.
- `engagement::RecurringEngagementPolicy` owns recommendation choice, not message sending.
- `training::FollowUpService` owns orchestration from decision to internal task/draft.
- Provider adapters own translation into provider calls and must require typed approvals before writes.

## 5. Review and approval contract

### Automation level

Allowed without human approval:

- Read package/enrollment/progress snapshots.
- Compute deterministic package balances from typed ledger entries.
- Detect mismatches, unused sessions, expired/expiring packages, missing payment, and lapsed engagement.
- Draft internal staff tasks and recommendation records.
- Draft parent-message text in a review queue with evidence citations and no send permission.
- Suppress outreach when policy says no safe action exists.

Staff review required:

- Sending any parent/customer training package, rebooking, homework, or outcome-related message.
- Confirming a re-enrollment recommendation is appropriate for the pet and customer.
- Scheduling package-backed sessions or class placements.
- Marking outcome/progress evidence as suitable for customer-facing use.
- Resolving routine provider ledger mismatches that do not affect refunds/credits/comps.

Trainer review required:

- Recommendations based on behavior goals, CGC readiness, reactivity, confidence, owner-handling plans, or sensitive training outcomes.
- Recommendations that change curriculum, package fit, or trainer requirement.
- Messages that explain progress, homework, behavior observations, or next-step training advice.

Manager approval required:

- Package extensions, credits, refunds, comps, transfers, or waived payment/deposit requirements.
- Overriding package eligibility, trainer capacity, class capacity, or waitlist constraints.
- Responding to complaints, incidents, safety-sensitive behavior, or disputed outcomes.
- Changing location package rules, pricing, expiration windows, or public availability.

Never automated:

- Capturing payment, issuing refund/credit, or modifying package balance in the provider without explicit approval.
- Sending customer/member-facing communications directly.
- Claiming guaranteed behavior outcomes or CGC readiness from AI-only evidence.
- Inventing missing session/progress evidence to justify an upsell.

### Audit trail

Every package/recurring-engagement action records:

- source event and timestamp;
- package/enrollment/session/customer/pet/location IDs;
- package policy version and recurring-engagement policy version;
- evidence references used for the decision;
- decision, review gate, and actor;
- before/after package balance for ledger mutations;
- provider draft/write ID and final provider result when applicable;
- suppression/blocker reasons when no action is taken.

### Customer/member-facing boundaries

Customer-facing drafts must carry `ApprovalBoundary::MemberFacingSendRequiresApproval` until a staff/trainer/manager approval event clears the gate. Package recommendations should be framed as optional next steps and availability-dependent, not guarantees. Sensitive behavior/care/payment facts should be routed to staff review and not appear verbatim in draft messages unless a human approves the exact language.

## 6. Test contracts

Later implementation should add semantic tests before code changes. Suggested names:

### Package ledger tests

- `training_package_ledger_balance_counts_purchased_reserved_consumed_and_available_sessions`
  - A ledger with purchases/reservations/completions exposes typed `SessionBalance` and never requires callers to recompute counts.
- `training_package_ledger_rejects_consumption_beyond_available_sessions_without_manager_adjustment`
  - Consuming a session beyond purchased units returns a semantic package error or reconciliation decision.
- `training_package_reservation_is_released_when_session_is_cancelled_before_completion`
  - Cancelled/no-show behavior changes reserved units according to package policy rather than silently consuming them.
- `training_package_manager_adjustment_requires_manager_approval`
  - Credit/refund/extension/overrun entries cannot be created with staff-only or agent approval.
- `training_package_provider_reference_cannot_be_used_as_domain_package_id`
  - Domain package ID and provider ref are distinct types and cannot be swapped.

### Eligibility and payment tests

- `training_package_eligibility_blocks_wrong_location_wrong_pet_or_wrong_program_kind`
  - Package eligibility returns typed blockers, not boolean false.
- `training_package_payment_required_blocks_schedule_confirmation_without_payment_authorization_draft`
  - Payment/package state is explicit and provider payment work remains a draft/approval boundary.
- `training_board_and_train_bundle_requires_reservation_context`
  - Board-and-train package use cannot be attached to a free-floating session when reservation context is required.
- `training_class_series_package_requires_matching_class_series_or_review_gate`
  - Group-class/package mismatch routes to review instead of consuming units.

### Recurring engagement tests

- `training_recurring_engagement_recommends_using_remaining_sessions_without_customer_send_permission`
  - Unused sessions produce staff tasks or message drafts with review gates.
- `training_recurring_engagement_suppresses_outreach_for_customer_opt_out_recent_complaint_or_manager_hold`
  - Suppression policy prevents draft sends and records reasons.
- `training_recurring_engagement_routes_behavior_sensitive_next_steps_to_trainer_review`
  - Reactivity/CGC/owner-handling recommendations require trainer approval before customer language.
- `training_recurring_engagement_does_not_recommend_re_enrollment_when_trainer_capacity_is_unavailable`
  - Capacity can suppress or route to manager review instead of creating a booking prompt.
- `training_completed_program_can_create_re_enrollment_consult_task_from_outcome_evidence`
  - Outcome evidence drives internal consult tasks without direct member-facing side effects.

### Workflow and audit tests

- `training_package_follow_up_creates_staff_task_with_training_specific_kind_and_review_gate`
  - Follow-up work does not hide training meaning in generic titles.
- `training_parent_message_draft_preserves_evidence_and_approval_boundary_across_serialization`
  - Storage roundtrip does not lose review gate/evidence links.
- `training_package_usage_reconciliation_records_provider_disagreement_without_mutating_ledger`
  - Provider mismatch is an audit/reconciliation finding until resolved.
- `training_agent_can_detect_unused_sessions_but_cannot_mark_parent_message_approved`
  - Agent output is a draft/recommendation only.
- `training_package_audit_event_records_policy_version_actor_review_gate_and_before_after_balance`
  - Ledger mutation audit is complete enough for manager review and debugging.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Short-term home for `operations::training::package` and `operations::training::engagement` modules if the crate remains single-file.
  - Prefer splitting into `domain/src/operations/training/{mod.rs,package.rs,engagement.rs,...}` when refactor scope permits; expose one canonical `operations::training` public surface.
- `domain/src/workflow.rs`
  - Add or refine draft/recommended-action types if current workflow packets cannot carry review gates/evidence.
- `domain/src/payment.rs` and/or existing payment module
  - Ensure training package logic consumes typed payment/deposit status rather than raw provider fields.
- `domain/src/entities.rs`
  - Only touch for missing typed identities or service-kind relationships; do not put package behavior on generic entities.
- `domain/tests/petsuites_training_package_contracts.rs`
  - Domain contract tests for package ledger, eligibility, recurring engagement, and approval boundaries.
- `domain/tests/training_agent_boundaries.rs`
  - Agent boundary tests for unused-session detection and message approval restrictions.
- `storage/tests/training_package_storage.rs`
  - Serialization/roundtrip tests after domain types are stable.

### Migration and refactor risks

- Existing `operations::training::PackagePolicy` is too small to represent ledger state. Do not overload it with counters; add `training::package::Ledger` and keep `PackagePolicy` as the rule shape.
- Current `FollowUpCadence` mixes timing with engagement intent. Introduce `training::engagement::Cadence`/`Decision` before implementing recommendation behavior.
- Package/session counts must remain semantic newtypes. Avoid raw `u16` arithmetic at call sites.
- Provider package balances may be incomplete or inconsistent. Store provider snapshots separately and require reconciliation before domain ledger mutation.
- Payment/refund/credit operations are adjacent-domain/provider actions, not training-core behavior. Training should emit drafts/review requirements.
- Member-facing language is sensitive because training can imply behavior guarantees or safety claims. Preserve `ApprovalBoundary` through serialization and workflow drafts.
- Board-and-train bundles cross reservation, boarding, trainer scheduling, and payment. Keep the training package decision explicit instead of hiding it inside reservation add-ons.

### Dependencies on other implications

- Enrollment/readiness implication: package eligibility depends on enrollment status, readiness blockers, care/behavior review, and reservation context.
- Trainer availability implication: recurring recommendations must honor trainer capacity, named-trainer requirements, and class calendars.
- Progress/outcome implication: re-enrollment candidates require trainer evidence, milestone state, and approved outcome documentation.
- Parent follow-up implication: package/re-enrollment messages should reuse the same approval-gated draft/send contract.
- Storage/serialization implication: ledger, audit, and approval-boundary state must roundtrip without losing semantic gates.

### Implementation sequencing recommendation

1. Add failing tests for `training::package::Id`, `SessionBalance`, `Ledger`, `LedgerEntry`, and `UsageDecision`.
2. Add package ledger methods and semantic errors without provider writes.
3. Add `PackageEligibilityPolicy` and payment/reservation/trainer blocker tests.
4. Add `training::engagement::{Candidate, Recommendation, Decision, SuppressionPolicy}` and staff-task/message-draft outputs.
5. Add audit events and serialization roundtrips.
6. Add agent specs/tools only after deterministic policies prove that agents cannot approve sends, writes, refunds, credits, or outcome claims.
