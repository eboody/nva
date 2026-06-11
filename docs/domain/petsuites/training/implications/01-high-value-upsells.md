# Training implication 01: High-value upsells

Purpose: define the domain contract for safe, high-value Training upsell recommendations in the PetSuites operations foundation. This is a modeling artifact for later Rust/domain cards, not an implementation patch. It assumes AI may detect and draft opportunities, while deterministic policies and staff review own approval, customer-facing language, and provider mutations.

## 1. Operational story

### Trigger

A high-value Training upsell opportunity can be considered when one or more of these operational facts appears:

- A boarding or daycare reservation includes a pet with a behavior goal, parent question, prior training interest, or staff-observed need that fits a Training offering.
- A Training enrollment completes, reaches a milestone, leaves unused package sessions, or is due for `operations::training::FollowUpCadence` outreach.
- A pet has progress evidence or an outcome suggesting a next-step program: private lesson, CGC prep, puppy kindergarten, tutor session, Stay and Study extension, or multi-session package.
- A customer asks about behavior, puppy skills, leash walking, reactivity, social confidence, or CGC readiness through a `TrainingOptionsQuestion` workflow.
- A manager daily brief detects lapsed training engagement, package underuse, trainer capacity openings, or cross-service context such as boarding plus tutor-session availability.

The trigger is a candidate signal, not permission to sell, book, charge, or send a message.

### Actors

- Customer / pet parent: receives only staff-approved recommendations and chooses whether to buy or book.
- Pet: the training subject; eligibility, behavior facts, care restrictions, and progress evidence are typed inputs.
- Trainer: reviews fit, capacity, curriculum suitability, outcome claims, homework, and sensitive behavior wording.
- Resort manager: approves capacity overrides, pricing/package exceptions, escalations, refunds/credits, and policy-sensitive customer commitments.
- Front-desk / customer-care staff: handles approved outreach, booking drafts, package explanation, and follow-up tasks.
- AI operations agent: summarizes evidence, detects candidate opportunities, drafts internal tasks/messages, and records why an opportunity needs review.
- Deterministic policy layer: evaluates eligibility, trainer capacity, package rules, approval boundaries, and audit requirements.

### Inputs

- Typed identities: `entities::CustomerId`, `entities::PetId`, `entities::LocationId`, optional `entities::ReservationId`, optional `operations::training::EnrollmentId`.
- Current service context: boarding/daycare reservation, active or completed Training enrollment, package state, trainer assignment, and location offerings.
- Training facts: `operations::training::ProgramKind`, `CurriculumMilestone`, `MilestoneStatus`, `ProgressEvidence`, `OutcomeDocumentation`, `FollowUpCadence`, and `PackagePolicy`.
- Pet/care facts: species, age/life-stage when available, care profile, medical/medication review state, temperament/behavior flags, and policy hard stops.
- Commercial context: prepaid sessions, remaining package sessions, payment/deposit readiness, location pricing/package rules, and cancellation/refund boundaries.
- Customer context: contact preference, opt-out/suppression state, recent complaints/escalations, and prior accepted/declined offers when available.
- Capacity snapshot: trainer availability, qualification requirements, class seats, named-trainer constraints, and waitlist state.

### Decisions

`operations::training::RevenueService` or a narrower `operations::training::upsell::Policy` should decide:

1. Is there enough typed evidence to create an upsell candidate?
2. Which `UpsellKind` truthfully matches the evidence and location offerings?
3. Is the pet/customer/reservation/enrollment eligible, blocked, or review-gated?
4. Is trainer capacity available, waitlisted, or manager-review-only?
5. Is the recommendation safe as internal-only, staff-review-required, manager-review-required, or not allowed?
6. Should the output be a staff task, workflow draft, suppressed no-action decision, or audited declined opportunity?
7. What customer-facing promise boundary applies: recommendation only, booking draft, package quote draft, or no customer language until trainer review?

The policy must not treat a model confidence score as approval. Confidence can prioritize review queues, but cannot waive typed gates.

### Outputs

- `operations::training::UpsellOpportunity`: typed candidate with source evidence, recommended program/package, value rationale, review state, and expiration.
- `operations::training::UpsellDecision`: `Recommend`, `NeedsReview`, `Suppress`, or `Blocked`, with semantic reasons.
- `operations::StaffTask` or training-specific staff task kind: trainer review, manager review, outreach follow-up, package consultation, or outcome documentation cleanup.
- `workflow::DraftMessage` / `workflow::DraftTask` carrying an approval boundary and never pre-marked approved by an agent.
- Optional provider-tool draft for booking/package changes, gated behind staff/manager approval and payment policy.
- Audit entry tying the opportunity to evidence, policy version, actor, review gate, and final disposition.

### Success state

A successful high-value upsell is one of:

- A staff-approved recommendation is sent to the customer with accurate scope, no guarantee-like claims, and a clear next step.
- A trainer/manager approves a booking or package draft, then a provider tool records the change with payment/package state preserved.
- Staff deliberately suppresses the opportunity because evidence is weak, timing is wrong, capacity is unavailable, customer preferences disallow outreach, or safety/care context makes the offer inappropriate.

The domain success criterion is safe, explainable, audited handling of the opportunity. Revenue conversion is an outcome metric, not the invariant.

### Failure and exception states

- `InsufficientEvidence`: opportunity lacks progress, reservation, customer-interest, or staff-observation evidence.
- `EligibilityBlocked`: medical, vaccine, care-profile, temperament, species, or policy hard stop prevents recommendation or scheduling.
- `TrainerCapacityUnavailable`: no qualified trainer/class capacity exists for the proposed program or date range.
- `PaymentOrPackageReviewRequired`: deposit, package purchase, unused sessions, refund/credit, or comp boundary requires human approval.
- `MemberFacingApprovalMissing`: draft exists but cannot be sent because trainer/staff/manager approval is absent.
- `SensitiveBehaviorLanguage`: behavior, safety, incident, or outcome wording requires trainer or manager review.
- `CustomerSuppressed`: contact preference, opt-out, complaint, or recent decline prevents automated outreach.
- `LocationOfferingUnavailable`: recommended program/package is not active at the location.
- `StaleOpportunity`: capacity, reservation window, package balance, or follow-up cadence expired before review.
- `DuplicateOrConflictingOpportunity`: another open opportunity or staff task already covers the same pet/program/window.

## 2. Domain types to add or refine

Prefer semantic paths under `operations::training`. Keep existing `operations::RevenueOpportunityKind::TrainingConsultCandidate` as a coarse dashboard signal, but move detailed behavior into Training-owned types.

### Core aggregate and identifiers

- `operations::training::upsell::OpportunityId`
  - Stable ID for an audited upsell candidate; cannot be swapped with enrollment/reservation IDs.
- `operations::training::UpsellOpportunity`
  - Fields: `id`, `location_id`, `customer_id`, `pet_id`, optional `reservation_id`, optional `enrollment_id`, `kind`, `source`, `evidence`, `recommended_offer`, `value_rationale`, `decision`, `approval_boundary`, `expires_at`, `audit_ref`.
  - Invariant: every opportunity has at least one typed evidence item and one typed recommended offer, unless its decision is `Blocked` for cleanup/audit only.
- `operations::training::UpsellKind`
  - `IntroConsult`
  - `DaycareTutorSession`
  - `BoardingTutorSession`
  - `StayAndStudyProgram { duration: StayAndStudyDuration }`
  - `PrivateLessonPackage`
  - `GroupClassEnrollment`
  - `PuppyKindergartenEnrollment`
  - `CgcPrepPath`
  - `PostProgramContinuationPackage`
  - `UnusedSessionRecovery`
- `operations::training::RecommendedOffer`
  - Pairs an active `OfferingId` or `ProgramKind` with a package/payment expectation and trainer requirement.
  - Invariant: a location-unavailable offering cannot become a recommendable offer.

### Evidence and source vocabulary

- `operations::training::UpsellSource`
  - `ReservationContext { reservation_id, service_context }`
  - `CompletedTrainingOutcome { enrollment_id }`
  - `ProgressMilestone { enrollment_id, milestone_id }`
  - `UnusedPackageSessions { enrollment_id, remaining: SessionCount }`
  - `CustomerInquiry { workflow_event_id }`
  - `StaffObservation { task_id }`
  - `ManagerBriefDetection { brief_id }`
- `operations::training::UpsellEvidence`
  - `BehaviorGoal { goal: BehaviorGoal }`
  - `CurriculumProgress { milestone_id, status: MilestoneStatus }`
  - `OutcomeRecorded { outcome: Outcome }`
  - `ParentInterest { topic: TrainingInterestTopic }`
  - `PackageBalance { remaining: SessionCount }`
  - `CapacityOpening { requirement: TrainerRequirement }`
  - `CrossServiceFit { reservation_id, fit: CrossServiceTrainingFit }`
  - Invariant: AI-created evidence is a draft extraction until staff/trainer approval records it as source-of-truth.
- `operations::training::TrainingInterestTopic`
  - `PuppyBasics`, `LeashWalking`, `Recall`, `Confidence`, `ReactivitySupport`, `CgcReadiness`, `GeneralManners`, `OtherReviewed`.
- `operations::training::ValueRationale`
  - Non-empty, bounded internal text or structured enum explaining why the recommendation is high-value; never directly customer-facing.

### Decisions, blockers, and review gates

- `operations::training::UpsellDecision`
  - `Recommend { action: UpsellAction, review: UpsellReviewState }`
  - `NeedsReview { gates: Vec<TrainingReviewGate>, reasons: Vec<UpsellReviewReason> }`
  - `Suppress { reason: UpsellSuppressionReason }`
  - `Blocked { blockers: Vec<UpsellBlocker> }`
- `operations::training::UpsellAction`
  - `CreateStaffTask`, `DraftCustomerMessage`, `DraftBookingChange`, `DraftPackageQuote`, `ManagerBriefOnly`, `NoActionAuditOnly`.
- `operations::training::UpsellReviewState`
  - `InternalAutomationAllowed`, `TrainerReviewRequired`, `StaffApprovalRequired`, `ManagerApprovalRequired`, `MemberFacingSendRequiresApproval`, `ProviderMutationRequiresApproval`.
- `operations::training::UpsellBlocker`
  - `Readiness(ReadinessBlocker)`, `NoActiveOffering`, `NoTrainerCapacity`, `PaymentOrPackageRequired`, `CustomerContactSuppressed`, `DuplicateOpenOpportunity`, `SensitiveBehaviorReviewRequired`, `StaleEvidence`, `ReservationContextRequired`.
- `operations::training::UpsellSuppressionReason`
  - `RecentlyDeclined`, `AlreadyEnrolled`, `OfferNotRelevant`, `UnsafeOrInsensitiveTiming`, `CustomerPreference`, `ManagerSuppressed`, `EvidenceTooWeak`.

### Policies, stores, tasks, and agent specs

- `operations::training::upsell::Policy`
  - Owns eligibility, fit, suppression, and approval-boundary decisions for upsell opportunities.
- `operations::training::upsell::Repository`
  - Persists opportunities, decisions, audit refs, and duplicate-detection queries.
- `operations::training::offering::Repository`
  - Loads active offerings and package constraints by location.
- `operations::training::trainer::ScheduleRepository`
  - Provides availability/capacity snapshots; does not approve overrides.
- `operations::training::UpsellStaffTaskKind`
  - Training-specific task taxonomy to add before stuffing upsell meaning into generic staff-task titles.
- `agents::training::UpsellScoutSpec`
  - Agent spec/tool contract for candidate extraction and evidence summaries; returns drafts only.

## 3. Relationship map

### Entities and value objects

- `entities::CustomerId`, `entities::PetId`, `entities::LocationId`, `entities::ReservationId`, and `entities::StaffId` identify the actors and context.
- `operations::training::UpsellOpportunity` references those IDs plus `OfferingId`, optional `EnrollmentId`, `ProgramKind`, `PackagePolicy`, and `TrainerRequirement`.
- `operations::training::UpsellEvidence`, `ValueRationale`, `TrainingInterestTopic`, and `RecommendedOffer` are value objects. They carry meaning but do not own workflow mutation.

### Policies

- `operations::training::ReadinessPolicy` evaluates pet/customer/reservation/payment prerequisites.
- `operations::training::TrainerAvailabilityPolicy` evaluates capacity and named/qualified trainer requirements.
- `operations::training::upsell::Policy` composes readiness, offering availability, package state, evidence quality, suppression rules, and review boundaries into `UpsellDecision`.
- `operations::training::ProgressReportPolicy` and `RecurringEngagementPolicy` feed evidence and follow-up cadence into the upsell policy, but should not own booking or message approval.

### Repositories and stores

- `training::offering::Repository`: active sellable programs/packages by location.
- `training::enrollment::Repository`: current and prior enrollments, package balance, completion/outcome state.
- `training::progress::Repository`: milestone/evidence/outcome facts; distinguishes approved source-of-truth from agent extracts.
- `training::trainer::ScheduleRepository`: capacity snapshots and assignments.
- `training::upsell::Repository`: open opportunity lookup, save decision, mark reviewed/suppressed/expired, and audit trail lookup.
- Storage adapters should serialize typed enums/newtypes directly or through explicit codecs; no raw string opportunity kinds or approval booleans should leak into the domain core.

### Workflow events and staff tasks

- Candidate signals become `workflow::WorkflowEvent` or `operations::OperationalSignal::RevenueOpportunities` with `RevenueOpportunityKind::TrainingConsultCandidate` only as a coarse rollup.
- Detailed action should be represented as `training::UpsellOpportunity` plus `operations::StaffTask`/`workflow::DraftTask`.
- Add training-specific staff task kinds such as `TrainingUpsellReview`, `TrainingPackageConsult`, `TrainingOutcomeFollowUp`, and `TrainingCapacityReview` when implementation reaches task modeling.

### Agent specs and tools

- `agents::training::UpsellScoutSpec` may read approved facts and produce `UpsellCandidateDraft` with cited evidence.
- `agents::training::FollowUpDraftSpec` may draft internal/staff-review customer language from an approved opportunity.
- Provider tools may only receive booking/package/message mutation drafts after policy and human approval. Tool execution belongs to `tools`/workflow boundaries, not the upsell policy.

## 4. Interaction contract

Rust-like pseudo-signatures are illustrative. The key constraint is behavior ownership: evidence extraction belongs to agent specs, policy decisions belong to policy objects, persistence belongs to repositories, and provider/customer side effects belong to workflow/tools after review.

```rust
impl operations::training::UpsellOpportunity {
    pub fn builder() -> upsell::OpportunityBuilder;
    pub fn requires_member_facing_approval(&self) -> bool;
    pub fn is_actionable_by_staff(&self) -> bool;
    pub fn expire(self, at: time::DateTime) -> Self;
}
```

```rust
pub trait operations::training::upsell::Policy {
    fn evaluate(
        &self,
        context: upsell::EvaluationContext,
    ) -> operations::training::UpsellDecision;
}

pub struct operations::training::upsell::EvaluationContext {
    pub location_id: entities::LocationId,
    pub customer_id: entities::CustomerId,
    pub pet_id: entities::PetId,
    pub reservation: Option<reservation::ContextSnapshot>,
    pub enrollment: Option<operations::training::EnrollmentSnapshot>,
    pub approved_evidence: Vec<operations::training::UpsellEvidence>,
    pub candidate_offer: operations::training::RecommendedOffer,
    pub customer_contact: customer::ContactPreferenceSnapshot,
    pub package_state: operations::training::PackageState,
    pub capacity: operations::training::TrainerCapacitySnapshot,
    pub now: time::DateTime,
}
```

Policy behavior:

- Return `Blocked` when readiness, offering availability, package/payment, contact suppression, or capacity makes the opportunity non-actionable.
- Return `NeedsReview` when evidence is plausible but trainer, staff, manager, behavior-sensitive, or member-facing approval is missing.
- Return `Recommend` only for internal task/draft creation unless review state explicitly allows staff action.
- Return `Suppress` when action would be irrelevant, duplicative, mistimed, or contrary to customer preference.

```rust
pub trait operations::training::upsell::Repository {
    fn find_open_for_pet(
        &self,
        pet_id: entities::PetId,
        kind: operations::training::UpsellKind,
    ) -> training::Result<Option<operations::training::UpsellOpportunity>>;

    fn save(
        &self,
        opportunity: &operations::training::UpsellOpportunity,
    ) -> training::Result<()>;

    fn mark_reviewed(
        &self,
        id: upsell::OpportunityId,
        review: operations::training::UpsellReviewOutcome,
    ) -> training::Result<()>;
}
```

```rust
impl operations::training::RevenueService {
    pub fn detect_high_value_upsell(
        &self,
        input: operations::training::upsell::DetectionInput,
    ) -> training::Result<operations::training::UpsellOpportunity>;

    pub fn create_review_task(
        &self,
        opportunity: &operations::training::UpsellOpportunity,
    ) -> training::Result<operations::StaffTask>;

    pub fn draft_follow_up(
        &self,
        opportunity: &operations::training::UpsellOpportunity,
    ) -> training::Result<workflow::DraftMessage>;
}
```

Service behavior:

- Load offerings, enrollment/progress, package state, and open duplicates through semantic repositories.
- Ask `ReadinessPolicy`, `TrainerAvailabilityPolicy`, and `upsell::Policy` for decisions; do not inline those rules as helper functions.
- Emit staff tasks and draft messages with explicit review gates.
- Never call provider mutation tools directly.

```rust
pub trait agents::training::UpsellScoutSpec {
    fn summarize_candidate(
        &self,
        facts: agents::training::ApprovedTrainingFacts,
    ) -> agents::training::UpsellCandidateDraft;
}
```

Agent contract:

- Input must be approved facts or explicitly marked draft facts.
- Output must cite evidence and carry `ApprovalBoundary::StaffReviewRequired` or stronger for member-facing use.
- Agent output cannot construct `UpsellDecision::Recommend` by itself; deterministic policy promotes or rejects it.

## 5. Review and approval contract

### Automation level

- Allowed automatically: candidate detection, duplicate checks, internal evidence summaries, manager-brief prioritization, creation of draft internal review tasks when policy permits.
- Staff/trainer review required: offer fit, behavior-sensitive rationale, progress/outcome claims, trainer suitability, homework/coaching language, customer-facing message text.
- Manager approval required: capacity overrides, named-trainer exceptions, package discounts/comps/refunds/credits, complaint/escalation contexts, sensitive behavior or safety implications, and policy/pricing exceptions.
- Never AI-only: booking confirmation, schedule changes, payment capture, refunds, package modifications, direct customer sends, guarantees, diagnoses, or CGC readiness claims.

### Review gates

Use existing or refined `operations::training::TrainingReviewGate` values:

- `TrainerReview`: validates program fit, evidence, curriculum next step, and trainer qualification.
- `ManagerApproval`: required for overrides, exceptions, escalations, and high-risk language.
- `ParentMessageApproval`: required before any customer-visible message leaves the system.
- `OutcomeDocumentationApproval`: required when the upsell depends on progress/outcome claims.
- `PaymentOrRefundApproval`: required for package, prepaid session, quote, credit, refund, or comp changes.
- `SafetyOrBehaviorReview`: required for reactivity, bite/aggression, medical/care concerns, or sensitive behavior labels.

### Audit trail

Every opportunity should record:

- Trigger/source and typed evidence IDs.
- Policy version and location policy/config snapshot.
- Recommended offer, package/payment expectation, and trainer requirement.
- Decision, blockers/suppression reasons, review gates, and expiration.
- Actor who approved, suppressed, sent, booked, or overrode.
- Draft/customer-message ID and provider mutation ID only after approved execution.

Audit must distinguish `agent drafted`, `policy recommended`, `staff approved`, `manager approved`, and `tool executed` as separate events.

### Customer/member-facing boundaries

- Customer language must be framed as an optional recommendation or next-step invitation, not a guarantee, diagnosis, or pressure tactic.
- Do not mention sensitive behavior/care facts unless staff approves exact wording.
- Do not imply capacity, price, package balance, or booking is final until provider state and payment policy are confirmed.
- Respect contact preferences and recent decline/suppression state.
- If the opportunity came from AI extraction, staff must review evidence before customer-facing use.

## 6. Test contracts

Later code cards should create failing semantic tests before implementation. Suggested tests:

- `training_upsell_opportunity_requires_typed_evidence_and_recommended_offer`
  - Constructing an actionable opportunity without evidence or an active recommended offer fails with a semantic error.
- `training_upsell_policy_blocks_when_location_does_not_offer_recommended_program`
  - A program unavailable at the location returns `UpsellDecision::Blocked { NoActiveOffering }`.
- `training_upsell_policy_routes_behavior_sensitive_rationale_to_trainer_or_manager_review`
  - Reactivity/safety evidence cannot produce a member-facing recommendation without trainer/manager review.
- `training_upsell_policy_suppresses_recently_declined_or_contact_suppressed_customers`
  - Contact preference and recent decline state prevent automatic outreach.
- `training_upsell_policy_recommends_internal_review_task_for_unused_package_sessions`
  - Remaining package sessions can create a staff follow-up task, but not a direct customer send.
- `training_upsell_policy_waitlists_or_reviews_when_required_trainer_capacity_is_unavailable`
  - No qualified/named trainer capacity yields waitlist or manager-review decision, not confirmed booking.
- `training_upsell_customer_message_draft_preserves_approval_boundary_across_serialization`
  - Storage roundtrip retains `MemberFacingSendRequiresApproval` / review gates.
- `training_upsell_agent_candidate_cannot_mark_policy_decision_recommended`
  - Agent draft output remains evidence/candidate only; deterministic policy owns decision promotion.
- `training_upsell_duplicate_open_opportunity_blocks_new_candidate_for_same_pet_kind_window`
  - Duplicate detection prevents staff-task spam.
- `training_upsell_audit_records_agent_policy_staff_and_tool_events_separately`
  - Audit trail preserves actor/source boundaries.
- `training_upsell_revenue_service_creates_staff_task_not_provider_mutation_for_recommended_offer`
  - Revenue service emits review workflow outputs, not direct booking/payment writes.
- `training_upsell_value_rationale_is_internal_until_parent_message_approval`
  - Internal rationale cannot be used as customer text without approval.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

Domain core:

- `domain/src/operations.rs`
  - Current training module lives here. Early implementation may add `training::upsell` submodule inline, but this file is already broad; consider splitting `domain/src/operations/training.rs` or `domain/src/operations/training/{mod.rs,upsell.rs}` if the codebase is ready for module extraction.
  - Add/refine `operations::training::{UpsellOpportunity, UpsellKind, UpsellDecision, UpsellEvidence, RecommendedOffer, UpsellReviewState, UpsellBlocker, UpsellSuppressionReason}`.
  - Add `RevenueService` / `upsell::Policy` contracts only after test shape is clear.
- `domain/src/agents.rs` or a later `domain/src/agents/training.rs`
  - Add draft-only `UpsellScoutSpec`/packet types if agent contracts are modeled in this crate.
- `domain/src/workflow.rs`
  - Ensure draft task/message contracts can carry training approval boundaries without raw booleans.
- `domain/src/policy.rs`
  - Reuse or refine generic `AutomationLevel`/`ReviewGate` only where it preserves training meaning; do not collapse training gates into vague generic flags.

Tests:

- `domain/tests/petsuites_training_upsell_contracts.rs`
- `domain/tests/training_agent_boundaries.rs`
- `domain/tests/training_relationship_contracts.rs`
- Storage serialization tests later, after the domain API stabilizes.

Storage / serialization later:

- Storage crate records/codecs for opportunities, decisions, review states, evidence, and audit events.
- Migration scripts/tables for training upsell opportunities only after domain semantics are stable.

### Migration and refactor risks

- `operations.rs` is currently a large file with multiple service modules. Adding more inline types may worsen discoverability; splitting must preserve canonical public paths such as `operations::training::UpsellOpportunity`.
- Existing `operations::RevenueOpportunityKind::TrainingConsultCandidate` is too coarse for high-value upsells. Do not remove it prematurely; use it as a rollup while adding detailed training-owned opportunity types.
- Existing `operations::training::PackagePolicy` has package shape but not package balance, quote, discount, refund, or approval semantics. Avoid overloading it with payment behavior that belongs to `payment`/`money` or manager approval workflows.
- Existing `ProgressTracking` is a strategy enum, not evidence. Add `ProgressEvidence`/`OutcomeDocumentation` before upsell rules depend on progress facts.
- Avoid raw strings for opportunity kind, reason, source, approval state, package state, or behavior topic. Use enums/newtypes and explicit codecs.
- Do not let agent specs create approved customer messages or provider tool calls. Boundary types should make this impossible or at least test-failing.
- Duplicate detection requires a semantic window/key, not just `pet_id`: include opportunity kind, recommended offer, reservation/enrollment context, and expiration.

### Dependencies on other implications

- Depends on Training enrollment/readiness modeling for blockers and review gates.
- Depends on progress/outcome documentation modeling for evidence-backed continuation and CGC recommendations.
- Depends on trainer availability/capacity modeling for waitlist and named/qualified trainer decisions.
- Depends on package/payment modeling for prepaid sessions, quotes, deposits, refunds, credits, and package exceptions.
- Depends on workflow/staff-task modeling for review tasks and approval-gated drafts.
- Can be implemented before provider tool execution because this implication should only create audited opportunities and review drafts.

### Recommended implementation sequence

1. Add domain tests for `UpsellKind`, `UpsellEvidence`, `RecommendedOffer`, and `UpsellOpportunity` construction invariants.
2. Add `UpsellDecision`, blockers, suppression reasons, and review states with serialization roundtrip tests.
3. Add `upsell::Policy` with pure deterministic fixtures for offering availability, contact suppression, capacity, package state, and evidence quality.
4. Add `training::RevenueService` orchestration that creates staff tasks/drafts but no provider mutation.
5. Add agent draft packet types only after policy tests prove agent output cannot approve itself.
6. Add storage codecs and migrations once the domain API is stable.
