# Grooming implication 04: Rebooking cadence every 2-8 weeks

Purpose: preserve a future design exploration for a PetSuites grooming cadence model. This is not a current runtime contract or implemented capability.

**Current executable status: unavailable.** Current serialized grooming/retention evidence is always suppressed and ineligible. It creates no candidate, ranking, queue, internal task, customer draft, schedule proposal, contact, conversion, completion, or value authority. Every proposed recommendation, task, draft, outreach, scheduling, approval-token, or agent behavior below is hypothetical future design only. It requires a new opaque, non-serializable eligibility authority issued after accepted source and consent review. Human review of current evidence cannot mint that authority.

Future-only assumption: if that opaque authority is designed and implemented, an ordinary recurring grooming recommendation could be constrained to 2-8 weeks, with explicit groomer, staff/manager, care/medical, customer-preference, and policy exceptions. Until then, agents may only preserve and summarize the suppressed evidence.

## 1. Future-only operational story

### Trigger

A future rebooking evaluation could start when one of these domain events or scheduled jobs appears, but none currently creates eligibility or work:

- A grooming appointment is completed and staff records the outcome/history.
- A nightly or daily retention scan evaluates pets whose last completed grooming service is approaching or past the recommended cadence.
- A groomer records a next-cadence recommendation during checkout, e.g. 4 weeks for a doodle with matting risk.
- A boarding/daycare checkout flow records exit-bath or grooming history as suppressed evidence; future opaque authority would be required before any follow-up draft could exist.
- A customer asks about next grooming timing and front desk needs a policy-backed recommendation.

The trigger produces a `operations::grooming::RebookingEvaluationRequested` workflow event or equivalent typed service call, not an immediate customer message.

### Actors

- Groomer: owns professional style/coat recommendation and can propose a pet-specific cadence inside or outside the ordinary band.
- Front desk / call center: currently inspects suppressed evidence only; a future authorized workflow could review recommendations and separately execute outreach or booking actions.
- Manager: approves exceptions involving no-show/deposit restrictions, cadence outside ordinary policy, discounts, complaints, or calendar overrides.
- Customer/member: receives only reviewed/approved outreach and confirms appointment details.
- Pet/customer/profile stores: provide identity, history, consent, communication preferences, and care/medical references.
- Grooming retention agent: currently has no candidate, ranking, task, or draft capability; a future opaque-authority design could define those capabilities.
- Provider tools: Gingr/calendar/messaging/POS adapters execute only approved commands.

### Inputs

- `entities::LocationId`, `entities::CustomerId`, `entities::PetId`.
- Last completed `operations::grooming::ServiceHistoryEntry`, including completion date, offering, outcome, groomer, style note reference, product/care flags, and any next-cadence recommendation.
- Location `operations::grooming::Contract` and `RebookingPolicy` snapshot.
- Optional `operations::grooming::CadencePreference` from customer or groomer history.
- Customer communication consent/channel preferences from `customer`.
- Pet profile and care/medical/handling references from `pet`/`care`.
- No-show/late-cancellation/deposit state from grooming/reservation/payment policy surfaces.
- Calendar capacity summary, only in a future opaque-authority workflow that explicitly permits schedule proposals.
- Current business date/time in the location timezone.

### Decisions

The rebooking workflow decides:

1. Is there a completed grooming history entry recent enough to be the cadence anchor?
2. What cadence source owns the recommendation: contract default, groomer recommendation, customer preference, service/coat policy, or manager override?
3. Is the recommended interval within the ordinary 2-8 week band?
4. Is the pet not-yet-due, due soon, due now, overdue, lapsed, policy-blocked, or unknown because history/profile data is incomplete?
5. Does the recommendation require groomer/staff/manager/care review before outreach?
6. Is customer contact permitted for this channel and message purpose?
7. Does future opaque authority exist? In the current runtime the answer is no, so the output is suppressed evidence and no action; future design could separately model a staff task, customer draft, schedule proposal, or manager item.

### Future-only outputs, currently unavailable

- Current output: suppressed, ineligible serialized evidence with no candidate, ranking, queue, task, draft, or schedule proposal.
- Future `operations::grooming::RebookingRecommendation`, internal-task, customer-draft, and schedule-proposal types require a new opaque, non-serializable eligibility authority and are not executable today.
- `entities::AuditEvent` recording input snapshot IDs, actor, policy version, decision, and approval token if one later exists.
- No provider-system mutation by default.

### Success state

A successful current run preserves the evidence as suppressed and ineligible without creating work. The following states describe a future opaque-authority workflow only:

- Not-yet-due pets produce no member-facing action and optionally a future evaluation date.
- Due/overdue labels remain evidence only today; a future authorized workflow could produce a staff-reviewable follow-up or customer draft.
- A customer-approved booking path eventually links the recommendation to an appointment request/plan and records the accepted cadence in service history.
- The audit trail explains why the interval was chosen and who approved any customer-facing or provider-mutating action.

### Failure and exception states

- Missing service history: preserve an `InsufficientHistory` evidence label; do not create a task or invent a cadence in the current runtime.
- Missing pet profile/coat/service facts: return `NeedsStaffReview` or `NeedsGroomerReview` with a typed reason.
- Cadence outside 2-8 weeks: reject as ordinary cadence; accept only as `GroomerRecommendedOutsideOrdinaryBand` or `ManagerOverride` with actor/evidence.
- Repeated no-show/late cancellation/deposit restriction: route to manager review or deposit-required decision; do not draft casual outreach that hides the restriction.
- Customer communication consent unavailable/denied: preserve suppression evidence only; no internal task, customer draft, or send.
- Calendar unavailable: recommendation can remain due/overdue, but concrete schedule windows are omitted or marked unavailable.
- Care/medical-sensitive product or handling context: route through care/groomer review; do not tell the customer medical suitability.
- Conflicting customer preference versus groomer policy: preserve both as typed evidence and route for staff resolution.
- Provider ID/date parse failure: fail at the boundary conversion into semantic errors before policy runs.

## 2. Future-only domain types requiring opaque eligibility authority

None of the types, actions, repositories, events, tools, signatures, or tests below exists as current executable retention authority. Names that resemble present code are design sketches and must not be wired to serialized evidence.

### Cadence values and invariants

- `operations::grooming::CadenceWeeks`
  - Canonical grooming cadence week value.
  - Invariant: positive integer.
  - Ordinary recurring invariant: 2-8 weeks; encode this through `OrdinaryCadenceWeeks` or `RebookingCadence::OrdinaryEvery(OrdinaryCadenceWeeks)` instead of relying on comments.
  - Refactor risk: current code has both `operations::CadenceWeeks` and `operations::grooming::CadenceWeeks`; choose one canonical owner for grooming policy.

- `operations::grooming::OrdinaryCadenceWeeks`
  - Newtype for the normal 2-8 week band.
  - Invariant: `2 <= weeks <= 8`.
  - Constructor error should distinguish `TooSoonForOrdinaryCadence { weeks }` from `TooLongForOrdinaryCadence { weeks }` so tests assert the business rule, not only failure.

- `operations::grooming::RebookingCadence`
  - Refine existing enum from `EveryWeeks(CadenceWeeks) | AsNeeded | GroomerRecommended` into a shape that preserves source and exception semantics:

```rust
enum RebookingCadence {
    OrdinaryEvery(OrdinaryCadenceWeeks),
    GroomerRecommended {
        interval: CadenceWeeks,
        groomer: entities::StaffId,
        rationale: CadenceRationale,
    },
    CustomerPreferred {
        interval: CadenceWeeks,
        preference: customer::PreferenceRef,
    },
    ManagerOverride {
        interval: CadenceWeeks,
        approval: policy::ApprovalToken,
        rationale: CadenceRationale,
    },
    AsNeeded,
    UnknownRequiresReview,
}
```

- `operations::grooming::CadenceRationale`
  - Enum or bounded note type for `CoatMaintenance`, `MattingPrevention`, `SkinProductFollowUp`, `StyleMaintenance`, `SeasonalShedding`, `CustomerPreference`, `NoShowRecovery`, `Other(extension::Label)`.
  - Invariant: free-form extension labels are trimmed/non-empty/bounded and may need redacted debug if customer/pet-specific.

### History and recommendation entities

- `operations::grooming::ServiceHistoryEntry`
  - Completed-service anchor for rebooking.
  - Required fields: entry ID or provider ref, location, customer, pet, completed date/time, offering, groomer/staff ref, outcome, optional next cadence recommendation, source appointment ref.
  - Invariant: rebooking policy only uses completed service entries, not drafts/cancelled/no-shows as positive anchors.

- `operations::grooming::LastGroomingService`
  - Value object extracted from history for policy evaluation.
  - Invariant: exactly one authoritative completed anchor for a pet/location/policy snapshot; ties require staff review.

- `operations::grooming::RebookingRecommendation`
  - Entity/value object representing the policy output.
  - Required fields: location, customer, pet, anchor history entry, cadence, due window, status, rationale, evidence, review requirement, automation boundary.
  - Invariant: cannot be constructed with a member-facing action boundary that exceeds the review/consent decision.

- `operations::grooming::DueWindow`
  - `starts_on`, `target_on`, `expires_on` dates in location calendar semantics.
  - Invariant: `starts_on <= target_on <= expires_on`; target derives from last service date + cadence interval unless explicitly overridden.

- `operations::grooming::RebookingStatus`
  - `NotYetDue`, `DueSoon`, `DueNow`, `Overdue`, `Lapsed`, `Blocked`, `InsufficientHistory`, `NeedsReview`.
  - Avoid booleans like `is_due`/`is_overdue` as primary domain state.

- `operations::grooming::RebookingEvidence`
  - Typed evidence list: history snapshot ID, contract version, last service date, cadence source, consent decision, no-show decision, calendar summary ref, agent confidence.
  - Invariant: agent confidence is evidence, not permission.

### Policies, workflow events, tasks, and agent contracts

- `operations::grooming::RebookingPolicy`
  - Owns conversion from history + cadence + today into `RebookingRecommendation`.
  - Invariant: ordinary cadence band is enforced here or in the cadence type constructor, not in UI/query code.

- `operations::grooming::RebookingReviewRequirement`
  - `None`, `FrontDeskReview`, `GroomerReview`, `ManagerReview`, `CareReview`, `ConsentRequired`.
  - Composable if multiple gates apply.

- `operations::grooming::RebookingActionBoundary`
  - `NoAction`, `InternalTaskOnly`, `DraftCustomerMessage`, `DraftScheduleProposal`, `ApprovedForCustomerSend(policy::ApprovalToken)`, `ApprovedForBookingTool(policy::ApprovalToken)`, `Disallowed`.

- `workflow::WorkflowEventType::GroomingRebookingEvaluationRequested`
  - Later event variant or typed event payload. Should reference `pet`, `customer`, `location`, and optional history anchor; never carry free-form provider JSON into the domain core.

- `operations::StaffTaskKind::GroomingRebookingReview`
  - Add only if `CustomerFollowUp` is too generic for later behavior. Fields should include pet/customer, recommendation ID, and reason.

- `agents::grooming_rebooking::PromptPacket` / `AgentSpec`
  - Typed packet containing recommendation context and allowed output schema.
  - Agent output should be `RebookingRecommendationDraft` or `RebookingAgentFinding`, not a tool command.

## 3. Future-only relationship map between types

### Entities

- `operations::grooming::ServiceHistoryEntry` anchors cadence and belongs to one `entities::PetId`, `entities::CustomerId`, and `entities::LocationId`.
- `operations::grooming::RebookingRecommendation` is derived from a service history entry plus policy snapshot; it may later link to an `operations::grooming::AppointmentRequest` or `AppointmentPlan`.
- `entities::AuditEvent` records every recommendation creation, review, approval, message send, and booking command.

### Value objects

- `OrdinaryCadenceWeeks`, `CadenceWeeks`, `DueWindow`, `CadenceRationale`, `RebookingEvidence`, `ConfidenceBasisPoints`, `RecommendationId`, `HistoryEntryId`.
- Customer channel/consent and pet care facts remain in neighboring modules and enter through typed snapshots/refs.

### Policies

- `RebookingPolicy` owns cadence/due status decisions.
- `NoShowPolicyEngine` contributes deposit/manager-review constraints.
- `ReminderPolicy` turns approved recommendations into message/reminder drafts.
- `AutomationPolicy` maps recommendation risk + consent + approval into allowed action boundary.
- `SchedulingPolicy` may rank windows only after a recommendation exists; it does not own the cadence rule.

### Repositories and stores

- `operations::grooming::HistoryRepository`: loads completed service history anchors and appends accepted cadence outcomes.
- `operations::grooming::RecommendationRepository`: persists recommendations, statuses, evidence, and audit refs.
- `operations::grooming::ContractRepository`: loads location contract/policy snapshots.
- `customer::ConsentRepository` or equivalent: supplies contact permission; grooming only consumes the decision.
- `operations::grooming::CalendarRepository`: optional schedule proposal support; final booking still needs approval.
- Storage adapters keep provider IDs/records in `storage::operations::*Record` and promote to semantic domain types before policy use.

### Workflow events

- `GroomingServiceCompleted` or completion import -> `ServiceHistoryEntry` -> `RebookingEvaluationRequested`.
- `RebookingRecommendationCreated` -> staff task or message draft.
- `RebookingRecommendationReviewed` -> approval token or requested changes.
- `RebookingCustomerOutreachApproved` -> messaging tool command.
- `GroomingAppointmentBookedFromRebooking` -> appointment/history link.

### Staff tasks

- `GroomingHistoryReview`: resolve missing/conflicting service history.
- `GroomingRebookingReview`: approve cadence/rationale/outreach.
- `GroomingNoShowManagerReview`: decide deposit or restriction handling before rebooking.
- `CustomerFollowUp { reason: RebookingDue }` can be a short-term reuse only if the reason enum is made semantically truthful.

### Agent specs and tools

- Agent spec: `agents::grooming_rebooking::Spec` with allowed actions `ExtractStructuredData`, `RankRecommendations`, `DraftCustomerMessage`, `CreateInternalTaskDraft`.
- Forbidden actions: booking/rescheduling/cancelling appointments, changing invoices/deposits, sending customer messages, waiving no-show/deposit restrictions.
- Tool ports: `tools::MessageDraftStore`, `tools::InternalTaskDraftStore`, `tools::CalendarAvailabilityRead`, `tools::AppointmentBookingCommand` gated by approval.

## 4. Future-only interaction contract

Rust-like pseudo-signatures below name the intended owners. Exact modules may change, but behavior should stay on semantic policies/repositories/services rather than free functions.

```rust
impl operations::grooming::OrdinaryCadenceWeeks {
    pub const MIN: u8 = 2;
    pub const MAX: u8 = 8;

    pub fn try_new(weeks: u8) -> operations::grooming::cadence::Result<Self>;
    pub fn get(self) -> u8;
}
```

```rust
impl operations::grooming::RebookingPolicy {
    pub fn evaluate(
        &self,
        request: operations::grooming::RebookingEvaluationRequest,
        history: operations::grooming::ServiceHistorySnapshot,
        contract: operations::grooming::Contract,
        today: location::LocalDate,
    ) -> operations::grooming::Result<operations::grooming::RebookingRecommendation>;
}
```

Behavior:

- Select the authoritative completed history anchor.
- Resolve cadence source using this precedence: approved manager override, explicit groomer recommendation, customer preference if policy-compatible, service/coat policy, contract default.
- Reject ordinary intervals outside 2-8 weeks at construction time.
- Compute `DueWindow` from anchor date and cadence.
- Classify status from `today` against due window.
- Attach review reasons for unknown history, missing profile, no-show restriction, care-sensitive scenario, outside-band exception, or member-facing boundary.

```rust
impl operations::grooming::HistoryRepository {
    pub async fn latest_completed_service(
        &self,
        location: entities::LocationId,
        pet: entities::PetId,
    ) -> operations::grooming::Result<Option<operations::grooming::LastGroomingService>>;

    pub async fn append_rebooking_outcome(
        &self,
        outcome: operations::grooming::AcceptedRebookingOutcome,
    ) -> operations::grooming::Result<()>;
}
```

Repository behavior:

- Return semantic entries only; provider status strings must already be converted.
- Do not treat cancelled/no-show appointments as completed service anchors.
- Surface duplicate/conflicting latest entries as a domain error requiring staff review.

```rust
impl operations::grooming::RecommendationRepository {
    pub async fn save(
        &self,
        recommendation: operations::grooming::RebookingRecommendation,
        audit: entities::AuditEvent,
    ) -> operations::grooming::Result<operations::grooming::RecommendationId>;

    pub async fn mark_reviewed(
        &self,
        id: operations::grooming::RecommendationId,
        decision: operations::grooming::RebookingReviewDecision,
        audit: entities::AuditEvent,
    ) -> operations::grooming::Result<()>;
}
```

```rust
impl operations::grooming::AutomationPolicy {
    pub fn decide_rebooking_action(
        &self,
        recommendation: &operations::grooming::RebookingRecommendation,
        consent: customer::CommunicationConsentDecision,
        no_show: operations::grooming::NoShowDecision,
        approval: Option<policy::ApprovalToken>,
    ) -> operations::grooming::RebookingActionBoundary;
}
```

Automation behavior:

- `DraftCustomerMessage` is allowed only when consent is present and no review gate blocks drafting.
- `ApprovedForCustomerSend` requires a typed approval token and channel consent.
- `ApprovedForBookingTool` requires approval and a separate scheduling/booking decision.
- No-show/deposit or manager-review states downgrade to internal task/manager review.

```rust
impl operations::grooming::ReminderPolicy {
    pub fn draft_rebooking_message(
        &self,
        recommendation: &operations::grooming::RebookingRecommendation,
        customer: customer::CommunicationProfile,
    ) -> operations::grooming::Result<tools::MessageDraft>;
}
```

Reminder behavior:

- Message copy must say it is an invitation/recommendation, not a booked appointment.
- Price/deposit/care suitability claims require typed evidence and review.
- The draft retains recommendation ID, evidence refs, and required approval gates.

```rust
impl agents::grooming_rebooking::Spec {
    pub fn prompt_packet(
        &self,
        recommendation_context: operations::grooming::RebookingAgentContext,
        policy_context: workflow::PolicyContext,
    ) -> agents::AgentPromptPacket<operations::grooming::RebookingAgentContext>;

    pub fn validate_output(
        &self,
        output: agents::grooming_rebooking::Output,
    ) -> operations::grooming::Result<operations::grooming::RebookingAgentFinding>;
}
```

Agent behavior:

- Agent may explain rationale, rank urgency, and draft staff/customer text.
- Agent output cannot carry execution permission.
- Validation rejects missing evidence, untyped dates, unsupported cadence intervals, and member-facing commands disguised as recommendations.

## 5. Future-only review and approval contract

### Automation level

Default automation level: `policy::AutomationLevel::DraftOnly`.

Safe without human approval:

- Compute due/overdue state from trusted history and contract policy.
- Create internal recommendation records and audit events.
- Create internal staff-task drafts for review.
- Draft but not send customer copy when consent exists and no risk gate is present.

Requires front-desk or groomer review:

- First outreach to a customer about a due/overdue grooming rebooking.
- Any recommendation based on groomer style notes, coat condition, sensitive product use, or incomplete profile data.
- Message copy that interprets customer preference or prior service outcome.
- Converting a recommendation into a concrete appointment request or selected schedule window.

Requires manager review:

- Cadence outside the ordinary 2-8 week band unless already captured as a groomer recommendation permitted by local policy.
- Rebooking after repeated no-show/late-cancellation/deposit restriction.
- Waiving deposits, applying discounts, changing cancellation penalties, or promising price adjustments.
- Customer complaint, refund, injury/safety, legal, or reputational escalation.
- Sending member-facing outreach despite ambiguous consent/channel policy.

Disallowed for automation alone:

- Scheduling, rescheduling, or cancelling a grooming appointment in the provider system.
- Charging/waiving/refunding deposits or adding services/products to invoices.
- Sending customer messages.
- Telling a customer that a pet is medically suitable for a product or service.
- Editing or suppressing groomer/care/incident history to make rebooking easier.

### Audit trail

Every recommendation and review action should record:

- actor (`System`, `Agent`, `Staff`, `Manager`) and timestamp,
- location/customer/pet/recommendation identifiers,
- source history entry and policy/contract snapshot IDs,
- cadence source and interval,
- due status and review gates,
- consent decision reference,
- no-show/deposit decision reference,
- generated message/task draft IDs if any,
- approval token for any send/booking/tool execution.

### Customer/member-facing boundaries

Customer-facing copy must remain clear and non-committal until the customer/staff confirms booking: “Moose may be due for grooming around June 12 based on the last full groom” is safe as a reviewed draft; “Moose is booked” or “we added a full groom” is not. Any price, deposit, medical, or calendar promise requires the owning policy/tool result and approval token.

## 6. Future-only test contracts

Later code cards should start with semantic tests named like executable glossary entries.

### Cadence value tests

- `grooming_ordinary_rebooking_cadence_accepts_two_through_eight_weeks`
  - `OrdinaryCadenceWeeks::try_new(2)` and `try_new(8)` succeed.
- `grooming_ordinary_rebooking_cadence_rejects_one_or_nine_weeks_without_exception_source`
  - `try_new(1)` and `try_new(9)` fail with domain-specific errors.
- `grooming_rebooking_cadence_outside_ordinary_band_requires_groomer_or_manager_source`
  - Outside-band cadence can only be represented as `GroomerRecommended` or `ManagerOverride` carrying actor/evidence.
- `grooming_top_level_cadence_weeks_do_not_duplicate_grooming_policy_semantics`
  - If top-level `operations::CadenceWeeks` remains, it does not claim the grooming 2-8 invariant.

### Policy tests

- `grooming_rebooking_policy_marks_pet_due_from_last_completed_service_and_six_week_contract_cadence`
  - Last service + six-week contract default yields due status at the correct location date.
- `grooming_rebooking_policy_ignores_cancelled_or_no_show_appointments_as_completed_service_anchor`
  - Only completed history can anchor the next recommendation.
- `grooming_rebooking_policy_returns_insufficient_history_when_no_completed_service_exists`
  - No invented cadence or member-facing message.
- `grooming_rebooking_policy_routes_conflicting_latest_history_entries_to_staff_review`
  - Duplicate latest anchors produce a review-required decision.
- `grooming_rebooking_policy_preserves_groomer_recommended_cadence_rationale`
  - Groomer recommendation carries staff ID/rationale into the output.
- `grooming_rebooking_policy_classifies_not_due_due_soon_due_now_overdue_and_lapsed`
  - Status is a semantic enum, not a boolean.

### Approval and workflow tests

- `grooming_rebooking_recommendation_without_customer_consent_creates_internal_task_only`
  - No sendable draft appears without consent.
- `grooming_rebooking_after_repeat_no_show_requires_manager_review_before_outreach_or_booking`
  - No-show policy blocks casual outreach/booking.
- `grooming_rebooking_message_draft_states_recommendation_not_confirmed_booking`
  - Draft text cannot imply a booked appointment or added charge.
- `grooming_rebooking_automation_policy_never_sends_or_books_from_agent_output_alone`
  - Agent output stays draft/review until approval token exists.
- `grooming_rebooking_audit_event_records_policy_snapshot_history_anchor_and_approval_token`
  - Auditing preserves why the recommendation was made and who approved execution.

### Storage and boundary tests

- `grooming_history_record_promotes_provider_completed_status_to_service_history_entry`
  - Provider payload converts into semantic completed history before policy.
- `grooming_history_record_rejects_unparseable_completion_date_before_policy_runs`
  - Boundary parse failures do not leak raw strings into policy.
- `grooming_rebooking_recommendation_record_roundtrips_due_window_status_cadence_source_and_review_boundary`
  - Storage preserves domain shape.
- `grooming_rebooking_recommendation_record_rejects_member_facing_send_without_approval_token`
  - Persisted action boundary cannot overstate permission.

### Agent tests

- `grooming_rebooking_agent_returns_recommendation_with_evidence_and_review_boundary`
- `grooming_rebooking_agent_rejects_free_form_booking_command_as_output`
- `grooming_rebooking_agent_draft_message_carries_recommendation_id_and_required_reviews`
- `grooming_rebooking_agent_flags_missing_history_or_consent_instead_of_hallucinating_outreach`

## 7. Future-only integration notes

### Files likely touched

- `domain/src/operations.rs`
  - Short-term location for `operations::grooming` additions if the crate remains single-file.
  - Later refactor candidate: split into `domain/src/operations/grooming/mod.rs`, `cadence.rs`, `history.rs`, `rebooking.rs`, `automation.rs`, `repository.rs`.
- `domain/src/workflow.rs`
  - Add grooming rebooking event/action variants only when workflow behavior needs them.
- `domain/src/agents.rs` and/or `domain/src/agent.rs`
  - Add typed grooming rebooking prompt/output contracts.
- `domain/src/customer.rs`
  - Consume or refine communication consent/profile types; avoid grooming-owned duplicates.
- `domain/src/entities.rs`
  - Add audit subjects/actions or staff task source/subject refs if existing generic variants are insufficient.
- `domain/src/tools.rs`
  - Add message draft/calendar/booking command boundaries only as approved tool ports.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add contract tests for `OrdinaryCadenceWeeks`, `RebookingCadence`, and standard PetSuites six-week default.
- `domain/tests/domain_quality_patterns.rs`
  - Add doctrine tests proving cadence/status/review are semantic types, not bool/string pairs.
- Future storage crate tests, likely `storage/tests/core_service_contract_storage.rs` and `storage/tests/operations_storage_contracts.rs`, if recommendations/history become persisted records.

### Migration and refactor risks

- Duplicate cadence semantics: existing `operations::CadenceWeeks` and `operations::grooming::CadenceWeeks` can drift. Pick a canonical owner before adding policy behavior.
- Compatibility with existing serialized contracts: changing `RebookingCadence::EveryWeeks(CadenceWeeks)` to richer variants may require serde aliases/migrations for existing fixtures.
- Date semantics: due windows should use location-local dates, not naive UTC arithmetic that can shift reminders around midnight/timezones.
- History anchoring: imported provider statuses must distinguish completed, cancelled, no-show, and rescheduled; otherwise policy may recommend from bad anchors.
- Consent and outreach: message drafts must not become send commands by shape or naming. Keep `DraftCustomerMessage` separate from `SendCustomerMessage`.
- No-show/deposit policy: do not model deposit restrictions as a grooming-only boolean; integrate with payment/reservation policy types.
- Sensitive notes: style notes, care/medical refs, and behavior/handling facts need separate typed fields and redacted debug behavior.
- Agent evidence: LLM confidence should not be stored as approval; approval tokens come from deterministic review flow.

### Dependencies on other implications and domain slices

- Grooming service history/style-photo implication: provides the completed history entry and notes model needed to anchor cadence.
- Groomer calendar/appointment-duration implication: needed only when rebooking proposes concrete schedule windows.
- No-show/cancellation implication: required before rebooking can safely handle deposit/manager-review restrictions.
- Reminder/customer communication implication: required for consent-aware outreach drafts and reminder plans.
- Cross-sell/boarding-daycare implication: may feed rebooking opportunities but should not own the cadence calculation.
- Core storage/serialization slice: needed before persisted recommendations can round-trip due windows/status/review boundaries.

Implementation entry rule: write the failing semantic cadence tests first, verify RED, then add the smallest grooming-owned value/policy surface. Do not implement the 2-8 week rule as a UI filter, SQL fragment, CRM campaign threshold, or date helper; it is a grooming cadence invariant with explicit exception paths.
